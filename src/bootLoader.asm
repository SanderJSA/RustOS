	BITS 16

entry_point:
	mov ax, 07C0h
	add ax, 288
	mov ss, ax
	mov sp, 4096

	mov ax, 07C0h
	mov ds, ax

	mov si, text_string
	call print_string

	jmp $

	text_string db 'Welcome to our bootloader.', 0

print_string:
	mov ah, 0Eh
.loop:
	lodsb
	cmp al, 0
	je .done
	int 10h
	jmp .loop

.done:
	ret

	times 510-($-$$) db 0
	dw 0xAA55
