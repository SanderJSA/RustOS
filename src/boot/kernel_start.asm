    section .kernel_start
	[bits 32]

	[extern _start] 	; tell code that function exists
	call _start	; Call main function in kernel
	
	jmp $