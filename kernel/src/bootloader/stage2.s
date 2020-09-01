.section .stage2, "awx"
.intel_syntax noprefix

# Stage 2 sets up paging by identity mapping the first 2MB
# Enters long mode and moves kernel to its target address
# Calls _start in the kernel

#===========#
# Constants
#===========#

.equ pml4t,          0x1000
.equ pdpt,           pml4t + 0x1000
.equ pdt,            pdpt + 0x1000
.equ pt,             pdt + 0x1000

#=======================#
# Set up Protected mode
#=======================#

.code32
stage2 :
    mov ax, 0x10 # Set up data segments
    mov ds, ax   #
    mov es, ax   #
    mov ss, ax   #

#====================#
# Set up Page Tables
#====================#

    lea edi, [pml4t]                  # Set the destination index to pml4t.
    mov ecx, 0x1000                   # Clear all entries
    xor eax, eax                      #
    rep stosd                         #

    lea edi, [pml4t]                  # Set the destination index to pml4t.
    add edi, 511 * 8                  # Map last entry to the table itself
    mov dword ptr [edi], pml4t | 0x03 #

    # Identity map the first 2MB for kernel and VGA
    lea edi, [pml4t]                  # get address of PML4T
    mov cr3, edi                      # Set Paging entry point to pml4t's address
    mov ebx, pdpt | 0x03              # PML4T[0] = PDPT[0] with read and write properties on
    mov dword ptr [edi], ebx          #

    lea edi, [pdpt]                   # get address of PDPT
    mov ebx, pdt | 0x03               # PML4T[0] = PDPT[0] with read and write properties on
    mov dword ptr [edi], ebx          #

    lea edi, [pdt]                    # get address of PDT
    mov ebx, pt | 0x03                # PML4T[0] = PDPT[0] with read and write properties on
    mov dword ptr [edi], ebx          #

    lea edi, [pt]                     # get address of PT
    mov ebx, 0x03                     # Set page start and properties
    mov ecx, 512                      # Repeat for 512 entries
.BuildPages:
    mov dword ptr [edi], ebx          # Write page info
    add ebx, 0x1000                   # Go to next Page
    add edi, 8                        # Go to next Page Table entry
    loop .BuildPages                  # Repeat 512 times

#=============================#
# Protected mode to Long mode
#=============================#

    mov eax, cr4        # Enable PAE and global-page mechanism
    or eax, 0b10100000  #
    mov cr4, eax        #

    mov ecx, 0xC0000080 # Set Long Mode enabled bit in EFER register
    rdmsr               #
    or eax, 1 << 8      #
    wrmsr               #

    mov eax, cr0        # Enable paging by setting CR0.PG bit to 1
    or eax, 1 << 31     #
    mov cr0, eax        #

    lgdt [gdt64_descriptor]

    push 0x8
    lea eax, [init_lm]
    push eax
    retf

#==================#
# Set up Long mode
#==================#

.code64
init_lm:
    mov ax, 0                # Set up data segments
    mov ds, ax               #
    mov es, ax               #
    mov fs, ax               #
    mov gs, ax               #
    mov ss, ax               #

    mov rbp, 0x90000         # Set up stack
    mov rsp, rbp             #

    lea rsi, [_stage2_end]   # Move loaded kernel
    lea rdi, [_kernel_start] # To _kernel_start
    lea rcx, [_kernel_size]  # _kernel_size times
    rep movsd                #

    lea rax, [_start]
    call rax


#===========#
# 64bit GDT
#===========#

.code32
GDT64:                              # Global Descriptor Table (64-bit).
    gdt64_null:                     # The null descriptor.
        .quad 0
    gdt64_code:                     # The code descriptor.
        .word 0                     # Limit (low).
        .word 0                     # Base (low).
        .byte 0                     # Base (middle)
        .byte 0b10011010            # Access (exec/read).
        .byte 0b10101111            # Granularity, 64 bits flag, limit19:16.
        .byte 0                     # Base (high).
    gdt64_data:                     # The data descriptor.
        .word 0                     # Limit (low).
        .word 0                     # Base (low).
        .byte 0                     # Base (middle)
        .byte 0b10010010            # Access (read/write).
        .byte 0b00000000            # Granularity.
        .byte 0                     # Base (high).
    gdt64_end:

gdt64_descriptor:                   # The GDT-pointer.
    .word gdt64_end - GDT64 - 1     # Limit.
    .quad GDT64                     # Base.

#===============#
# Pad to sector
#===============#

    .org 510
    .word 0
