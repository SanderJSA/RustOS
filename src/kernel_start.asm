	[bits 32]
	
	[extern main] 	; tell code that function exists
	call main	; Call main function in kernel
	
	jmp $
