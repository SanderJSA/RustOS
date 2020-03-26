section .kernel_start

[extern _start] ; tell code that function exists
call _start	    ; Call main function in kernel
