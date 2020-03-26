;===========;
; Constants
;===========;

    KERNEL_ADDRESS equ 0x1000000

    pmap_len       equ 0x7F04
    pmap_end       equ 0x7F38
    pmap           equ 0x7F3C

    pml4t		   equ 0x1000
    pdpt		   equ pml4t + 0x1000
    pdt			   equ pdpt + 0x1000

    DATA_SEG       equ 0x10
    CODE_SEG       equ 0x8

;============;
; Set up MBR
;============;

    [ORG 0x07C00]  ; Location of our bootloader
    [BITS 16]  	   ; Real mode only supports 16 bits

	xor ax, ax     ; Reset segments
	mov ds, ax     ;
	mov es, ax     ;

	mov ax, 0x2401 ; enable A20 bit
	int 0x15 	   ;

;=============;
; Get ram map
;=============;

	xor	ebx, ebx			; set ebx to 0x00
	xor	si, si		        ; used here as a counter
	mov edi, pmap - 24	   	; our destination buffer
rammap:
	add di, 24
	mov eax, 0xE820		    ; BIOS command
	mov ecx, 24     	    ; Try to retrieve 24 bytes
	mov edx, 0x534D4150		; 'SMAP' signature
	mov [es:di+20], dword 1 ; Ask for valid ACPI 3
	int 0x15				;
	inc si					; add one to the length
	cmp ebx, 0      		; if last entry
	jne rammap				; continue to next task
	mov [pmap_len], si
	add di, 24
	mov [pmap_end], di

;=============;
; Load kernel
;=============;

	mov dl, 0x0	   ; Select 1st floppy disk
	mov dh, 0x0	   ; Head : 0
	mov ch, 0x0	   ; Cylinder 0
	mov cl, 0x2	   ; Sector starts at 1, kernel is at 2

	xor bx, bx     ; set lefthand part of 0x0:[ram end]
	mov es, bx     ;
	mov bx, [pmap_end] ; set righthand partof 0x0:[ram end]

readDrive:
	mov ah, 0x02   ; Read Sector From Drive
	mov al, 0x01   ; Read just one sector
	int 0x13	   ; Interrupt for low-level disk services
	jc readDrive   ; Try to read again if floppy drive failed

;=============================;
; Real mode to Protected mode
;=============================;

	cli                   ; Disable interrupts
	lgdt [gdt_descriptor] ; load GDT

	mov eax,cr0           ; Entering protected mode
	or eax,1              ;
	mov cr0,eax		      ;

	jmp CODE_SEG:init_pm

;=======================;
; Set up Protected mode
;=======================;

    [BITS 32]

init_pm :
    mov ax, DATA_SEG ; Set up data segments
	mov ds, ax       ;
	mov es, ax       ;
	mov fs, ax       ;
	mov gs, ax       ;
	mov ss, ax       ;

	mov ebp, 0x9000  ; Set up stack
	mov esp, ebp     ;

;====================;
; Set up Page Tables
;====================;

    mov edi, pml4t     ; Clear Page Table
    mov ecx, 0x4000    ;
    xor eax, eax       ;
    rep stosd          ;

    ; PML4T @ 0x1000
    mov eax, pdpt      ; PDP base address
    or eax, 0b11       ; P and R/W bits
    mov ebx, pml4t     ; PMPL4 base address
    mov [ebx], eax

    ; PDP @ 0x2000; maps 64Go
    mov eax, pdt        ; PD base address
    mov ebx, pdpt       ; PDP physical address
    mov ecx, 64         ; 64 PDP
    build_PDP:
        or eax, 0b11
        mov [ebx], eax
        add ebx, 0x8
        add eax, 0x1000 ; next PD page base address
        loop build_PDP

    ; PD @ 0x3000 (ends at 0x4000, fits below 0x7c00)
    ; 1 entry maps a 2MB page, the 1st starts at 0x0
    mov eax, 0x0        ; 1st page physical base address
    mov ebx, 0x3000     ; PD physical base address
    mov ecx, 512

    build_PD:
        or eax, 0b10000011 ; P + R/W + PS (bit for 2MB page)
        mov [ebx], eax
        add ebx, 0x8
        add eax, 0x200000  ; next 2MB physical page
        loop build_PD
    ; (tables end at 0x4000 => fits before Bios boot sector at 0x7c00)

;=============================;
; Protected mode to Long mode
;=============================;

    mov eax, cr4        ; Enable PAE
    or eax, 1 << 5      ;
    mov cr4, eax        ;

    mov eax, cr4        ; Enable global-page mechanism by setting CR0.PGE bit to 1
    or eax, 1 << 7      ;
    mov cr4, eax        ;

    mov eax, pml4t      ; Load CR3 with PML4 base address
    mov cr3, eax        ;

    mov ecx, 0xC0000080 ; Set Long Mode enabled bit in EFER register (address 0xC0000080)
    rdmsr               ;
    or eax, 1 << 8      ;
    wrmsr               ;

    mov eax, cr0        ; Enable paging by setting CR0.PG bit to 1
    or eax, (1 << 31)   ;
    mov cr0, eax        ;

    lgdt [gdt64_descriptor]

    jmp CODE_SEG:init_lm

;==================;
; Set up Long mode
;==================;

    [bits 64]

init_lm:
	mov rbp, 0x90000        ; Set up stack
	mov rsp, rbp            ;

	mov esi, [pmap_end]     ; Move loaded kernel
	mov edi, KERNEL_ADDRESS ; To KERNEL_ADDRESS
	mov ecx, 0x1000         ;
	rep movsd               ;

    call KERNEL_ADDRESS     ; call kernel

;===========;
; 32bit GDT
;===========;

    [BITS 16]

GDT:
	gdt_null :
   	dd 0x0
   	dd 0x0

 	gdt_code:
   	dw 0xffff	    ; Limit
   	dw 0x0			; Base
   	db 0x0			; Base
   	db 10011010b	; 1st flag, Type flag
   	db 11001111b	; 2nd flag, Limit
   	db 0x0			; Base

 	gdt_data:
   	dw 0xffff      
   	dw 0x0         
   	db 0x0
   	db 10010010b
   	db 11001111b
   	db 0x0
 	gdt_end:

gdt_descriptor :
   dw gdt_end - GDT - 1     ; 16-bit size
   dd GDT            		; 32-bit start address

;===========;
; 64bit GDT
;===========;

    [BITS 32]

GDT64:
    ;null;
    dq 0x0

    gdt64_code:
    dd 0x0
    db 0x0
    db 0b10011000
    db 0b00100000
    db 0x0

    gdt64_data:
    dd 0x0
    db 0x0
    db 0b10010000
    db 0b00000000
    db 0x0

gdt64_descriptor :
    dw $ - GDT64 - 1        ; 16-bit size
    dd GDT64                ; 32-bit start address

;================;
; Boot signature
;================;

    [bits 16]
    times 510 -($-$$) db 0	; Zero-fill the remaining 510 bytes
    dw 0xAA55 		        ; Boot signature
