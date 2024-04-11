.text
.globl main
main:
	pushq %rbp
	movq %rsp, %rbp
	movl $1234567890, %esi
	leaq fmt(%rip), %rdi
	movl $0, %eax
	callq printf
	movl $0, %eax
	leave
	ret
.type main, @function
.size main, .-main
/* end function main */

.data
.balign 8
fmt:
	.ascii "compiler output: %d\n"
	.byte 0
/* end data */

.section .note.GNU-stack,"",@progbits
