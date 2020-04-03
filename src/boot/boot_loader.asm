;===========;
; Constants
;===========;

    KERNEL_ADDRESS equ 0x100000

    pmap_len       equ 0x8000
    pmap_end       equ 0x8004
    pmap           equ 0x8008

    pml4t          equ 0x1000
    pdpt           equ pml4t + 0x1000
    pdt            equ pdpt + 0x1000
    pt             equ pdt + 0x1000

    DATA_SEG       equ 0x10
    CODE_SEG       equ 0x8

;============;
; Set up MBR
;============;

    [ORG 0x07C00]  ; Location of our bootloader
    [BITS 16]      ; Real mode only supports 16 bits

    xor ax, ax     ; Reset segments
    mov es, ax     ;
    mov ds, ax     ;

    mov ax,0x2402  ; Get A20 status
    int 0x15       ;
    cmp al, 1      ; test if A20 is enabled
    jz A20Enabled  ; If enabled skip

    mov ax, 0x2401 ; Else enable A20 bit
    int 0x15       ;
A20Enabled:

;=============;
; Get ram map
;=============;

    xor ebx, ebx            ; set ebx to 0x00
    xor si, si              ; used here as a counter
    mov edi, pmap - 24      ; our destination buffer
rammap:
    add di, 24
    mov eax, 0xE820         ; BIOS command
    mov ecx, 24             ; Try to retrieve 24 bytes
    mov edx, 0x534D4150     ; 'SMAP' signature
    mov [es:di+20], dword 1 ; Ask for valid ACPI 3
    int 0x15                ;
    inc si                  ; add one to the length
    cmp ebx, 0              ; if last entry
    jne rammap              ; continue to next task
    mov [pmap_len], si
    add di, 24
    mov [pmap_end], di

;=============;
; Load kernel
;=============;

    mov dl, 0x0        ; Select 1st floppy disk
    mov dh, 0x0        ; Head : 0
    mov ch, 0x0        ; Cylinder 0
    mov cl, 0x2        ; Sector starts at 1, kernel is at 2

    xor bx, bx         ; set lefthand part of 0x0:[ram end]
    mov es, bx         ;
    mov bx, [pmap_end] ; set righthand partof 0x0:[ram end]

readDrive:
    mov ah, 0x02       ; Read Sector From Drive
    mov al, 0x3F       ; Read 10 sectors
    int 0x13           ; Interrupt for low-level disk services
    jc readDrive       ; Try to read again if floppy drive failed

;=============================;
; Real mode to Protected mode
;=============================;

    cli                   ; Disable interrupts
    lgdt [gdt_descriptor] ; load GDT

    mov eax,cr0           ; Entering protected mode
    or eax,1              ;
    mov cr0,eax           ;

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

    mov edi, pml4t               ; Set the destination index to pml4t.
    mov ecx, 0x1000              ; Clear all entries
    xor eax, eax
    rep stosd

    ; Identity map the first 2MB for kernel and VGA
    mov edi, pml4t               ; get address of PML4T
    mov cr3, edi                 ; Set Paging entry point to pml4t's address
    mov DWORD [edi], pdpt | 0x03 ; PML4T[0] = PDPT[0] with read and write properties on

    mov edi, pdpt                ; get address of PDPT
    mov DWORD [edi], pdt | 0x03  ; PDPT[0] = PDT[0] with read and write properties on

    mov edi, pdt                 ; get address of PDT
    mov DWORD [edi], pt | 0x03   ; PDT[0] = PT[0] with read and write properties on

    mov edi, pt                  ; get address of PT
    mov ebx, 0x03                ; Set page start and properties
    mov ecx, 512                 ; Repeat for 512 entries
.BuildPages:
    mov DWORD [edi], ebx         ; Write page info
    add ebx, 0x1000              ; Go to next Page
    add edi, 8                   ; Go to next Page Table entry
    loop .BuildPages             ; Repeat 512 times

;=============================;
; Protected mode to Long mode
;=============================;

    mov eax, cr4        ; Enable PAE
    or eax, 1 << 5      ;
    mov cr4, eax        ;

    mov eax, cr4        ; Enable global-page mechanism by setting CR0.PGE bit to 1
    or eax, 1 << 7      ;
    mov cr4, eax        ;

    mov ecx, 0xC0000080 ; Set Long Mode enabled bit in EFER register
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
    mov ax, DATA_SEG        ; Set up data segments
    mov ds, ax              ;
    mov es, ax              ;
    mov fs, ax              ;
    mov gs, ax              ;
    mov ss, ax              ;

    mov rbp, 0x90000        ; Set up stack
    mov rsp, rbp            ;

    mov esi, [pmap_end]     ; Move loaded kernel
    mov edi, KERNEL_ADDRESS ; To KERNEL_ADDRESS
    mov ecx, 0x6000         ;
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
       dw 0xffff        ; Limit
       dw 0x0           ; Base
       db 0x0           ; Base
       db 10011010b     ; 1st flag, Type flag
       db 11001111b     ; 2nd flag, Limit
       db 0x0           ; Base
    gdt_data:
       dw 0xffff
       dw 0x0
       db 0x0
       db 10010010b
       db 11001111b
       db 0x0
    gdt_end:

gdt_descriptor :
   dw gdt_end - GDT - 1 ; 16-bit size
   dd GDT               ; 32-bit start address

;===========;
; 64bit GDT
;===========;

    [BITS 32]

GDT64:                   ; Global Descriptor Table (64-bit).
    .Null: equ $ - GDT64 ; The null descriptor.
    dw 0xFFFF            ; Limit (low).
    dw 0                 ; Base (low).
    db 0                 ; Base (middle)
    db 0                 ; Access.
    db 1                 ; Granularity.
    db 0                 ; Base (high).
    .Code: equ $ - GDT64 ; The code descriptor.
    dw 0                 ; Limit (low).
    dw 0                 ; Base (low).
    db 0                 ; Base (middle)
    db 10011010b         ; Access (exec/read).
    db 10101111b         ; Granularity, 64 bits flag, limit19:16.
    db 0                 ; Base (high).
    .Data: equ $ - GDT64 ; The data descriptor.
    dw 0                 ; Limit (low).
    dw 0                 ; Base (low).
    db 0                 ; Base (middle)
    db 10010010b         ; Access (read/write).
    db 00000000b         ; Granularity.
    db 0                 ; Base (high).

gdt64_descriptor:        ; The GDT-pointer.
    dw $ - GDT64 - 1     ; Limit.
    dq GDT64             ; Base.

;================;
; Boot signature
;================;

    [bits 16]
    times 510 -($-$$) db 0 ; Zero-fill the remaining 510 bytes
    dw 0xAA55              ; Boot signature
