	[BITS 16]		; Real mode only supports 16 bits
	[org 0x07C00]		; Location of our bootloader

	xor ax, ax
	mov ds, ax

	mov ax, 0x2401
	int 0x15 		; enable A20 bit

	
;-------------------------------; Load kernel to memory

	mov dl, 0x0		; Select 1st floppy disk
	mov dh, 0x0		; Head : 0
	mov ch, 0x0		; Cylinder 0
	mov cl, 0x2		; Sector starts at 1, kernel is at 2
	
	xor bx, bx		; Data buffer is at ES:BX
	mov es, bx
	mov bx, 0x1000		; 0x0:0x1000

readDrive:
	mov ah, 0x02		; Read Sector From Drive	 
	mov al, 0x01		; Read just one sector
	int 0x13		; Interrupt for low-level disk services
	jc readDrive		; Try to read again if floppy drive failed

;-------------------------------; Kernel loaded, change to protected mode

	cli
	lgdt [gdt_descriptor]

	mov eax,cr0		;
	or eax,1		;
	mov cr0,eax		; Entering protected mode
	
	jmp CODE_SEG:init_pm
	
	[BITS 32]

init_pm :
	mov ax, DATA_SEG
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
	mov ss, ax
	
	mov ebp, 0x90000
	mov esp, ebp
	
	call 0x1000
	jmp $	

;-------------------------------;

[bits 16]

GDT:
	gdt_null :
   	dd 0x0
   	dd 0x0

 	gdt_code:
   	dw 0xffff			;Limit
   	dw 0x0			;Base
   	db 0x0			;Base
   	db 10011010b			;1st flag, Type flag
   	db 11001111b			;2nd flag, Limit
   	db 0x0			;Base

 	gdt_data:
   	dw 0xffff      
   	dw 0x0         
   	db 0x0
   	db 10010010b
   	db 11001111b
   	db 0x0
 	gdt_end:

gdt_descriptor :
   dw gdt_end - GDT - 1       	;16-bit size
   dd GDT            		;32-bit start address

CODE_SEG equ gdt_code - GDT
DATA_SEG equ gdt_data - GDT

;-------------------------------;

	times 510 -($-$$) db 0	; Zero-fill the remaining 510 bytes
	dw 0xAA55 		; Boot signature
