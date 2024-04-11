use std::{
    fs::File,
    io::{self, BufRead, Write},
    process::Command,
};

fn main() -> io::Result<()> {
    let file_path = "test.riz";
    let mut file = File::open(file_path)?;

    let tokens = tokenize_file(&mut file);

    tokens.iter().for_each(|token| println!("{:?}", token));

    let parsed = tokens_to_iml(&tokens);
    {
        let mut file = File::create("out.qbe")?;
        file.write_all(parsed.as_bytes())?;
        Command::new("qbe")
            .args(["-o", "out.s", "out.qbe"])
            .status()
            .expect("error executing qbe");
        Command::new("cc")
            .args(["out.s", "-o", &file_path[0..file_path.len() - ".riz".len()]])
            .status()
            .expect("error executing cc");
    }

    println!("{}", parsed);

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    If,
    Else,
    While,
    Riz,
    IntLit,
    Delimiter,
    Result,
    Identifier,
    Unrecognized,
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    value: Option<String>,
}

fn tokenize_file(file: &mut File) -> Vec<Token> {
    io::BufReader::new(file)
        .lines()
        .filter_map(|line| line.ok())
        .flat_map(|line| tokenize_line(&line))
        .collect()
}

fn match_identifiers(buf: &mut String) -> Option<TokenType> {
    let token_type = match buf.as_str() {
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "while" => TokenType::While,
        "riz" => TokenType::Riz,
        "'" => TokenType::Delimiter,
        "result" => TokenType::Result,
        _ => TokenType::Unrecognized,
    };
    if token_type == TokenType::Unrecognized {
        if buf.parse::<u32>().is_ok() {
            return Some(TokenType::IntLit);
        }
        return None;
    }
    Some(token_type)
}

fn tokenize_line(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.next() {
        if c.is_ascii_alphanumeric() || c == '\'' {
            buf.push(c);

            while let Some(&i) = iter.peek() {
                if i.is_numeric() {
                    buf.push(i);
                    iter.next();
                } else {
                    break;
                }
            }

            if let Some(token_type) = match_identifiers(&mut buf) {
                tokens.push(Token {
                    token_type,
                    value: Some(buf.clone()),
                });
                buf.clear()
            }
        }
    }

    tokens
}

fn tokens_to_iml(tokens: &[Token]) -> String {
    let mut out = String::from("export function w $main() {\n@start\n");

    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        if token.token_type == TokenType::Result {
            if let Some(next_token) = iter.peek() {
                if next_token.token_type == TokenType::IntLit {
                    let temp = next_token.value.clone();
                    iter.next();
                    if let Some(delimiter_token) = iter.peek() {
                        if delimiter_token.token_type == TokenType::Delimiter {
                            if let Some(value) = temp {
                                out.push_str(
                                    format!("    call $printf(l $fmt, ..., w {value})\n").as_str(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    out.push_str("    ret 0\n}\n");
    out.push_str("data $fmt = { b \"compiler output: %d\\n\", b 0 }");

    out
}
