.section .stage1, "awx"
.intel_syntax noprefix
.global bootloader_start

# Stage 1 loads stage 2 and the kernel,
# Creates a memory map
# Enters protected mode and calls stage 2

#===========#
# Constants
#===========#

.equ BUFFER,         0x7E00

.equ MEMORY_MAP_LEN, 0x500
.equ MEMORY_MAP,     0x504

#============#
# Set up MBR
#============#

.code16                  # Real mode only supports 16 bits
bootloader_start:
    xor ax, ax           # Reset segments
    mov ds, ax           #
    mov es, ax           #
    mov ss, ax           #
    mov fs, ax           #
    mov gs, ax           #

    lea sp, [_stack_end] # Set up stack

    mov ax, 0x2402       # Get A20 status
    int 0x15             #
    cmp al, 1            # test if A20 is enabled
    jz A20Enabled        # If enabled skip

    mov ax, 0x2401       # Else enable A20 bit
    int 0x15             #
A20Enabled:

#=============#
# Load kernel
#=============#

    lea eax, [_stage2_size]  # Add size of stage2 botloader and kernel
    lea ebx, [_kernel_size]  #
    add ebx, eax
    shr ebx, 9               # Determine number of sectors to load

readBlock:
    lea si, [dap]            # Load DAP struct
    mov ah, 0x42             # Use LBA addressing
                             # dl starts off set to the correct device
    int 0x13                 # Interrupt for low-level disk services

    mov ax, [dap_lba]        # Start at next sector
    add ax, 1                #
    mov [dap_lba], ax        #

    mov ax, [dap_buffer_seg] # Update buffer offset accordingly
    add ax, 512 / 16         #
    mov [dap_buffer_seg], ax #

    sub ebx, 1               # Continue if there are still blocks left to load
    jnz readBlock            #

#================#
# Get memory map
#================#

    lea di, [MEMORY_MAP]          # Where our memory map will be
    xor bp, bp                    # Keep an entry count in bp

    xor ebx, ebx                  # Set up registers to get an E820 memory map
    mov edx, 0x0534D4150          # Place "SMAP" into edx
    mov eax, 0xe820               #

    mov dword ptr es:[di + 20], 1 # Force a valid ACPI 3.X entry
    mov ecx, 24                   # Ask for 24 bytes

    int 0x15

    jmp .jmpin

.e820lp:
    mov edx, 0x0534D4150          # Fix potentially trashed register
    mov eax, 0xe820               #

    mov dword ptr es:[di + 20], 1 # Force a valid ACPI 3.X entry
    mov ecx, 24                   # Ask for 24 bytes again

    int 0x15

    jc .e820f                     # Reached end of list

.jmpin:
    jcxz .skipent                 # Skip any 0 length entries
    cmp cl, 20                    # Got a 24 byte ACPI 3.X response?
    jbe .notext
    test byte ptr es:[di + 20], 1 # If so: is the "ignore this data" bit clear?
    je .skipent

.notext:
    mov ecx, es:[di + 16]         # Get type of region
    cmp ecx, 2                    # Only keep usable and reclaimable memory
    je .skipent                   # Eliminate type 2 region
    cmp ecx, 4                    #
    jge .skipent                  # Eliminate type 4 and 5 region

    mov ecx, es:[di + 8]          # Get lower uint32_t of memory region length
    or ecx, es:[di + 12]          # Or it with upper uint32_t to test for zero
    jz .skipent                   # If length uint64_t is 0, skip entry

    add bp, 1                     # Got a good entry: ++count, move to next storage spot
    add di, 24                    # Update entry offset in memory map
.skipent:
    test ebx, ebx                 # If ebx resets to 0, list is complete
    jne .e820lp

.e820f:
    mov [MEMORY_MAP_LEN], bp      # Save the entry count

#=============================#
# Real mode to Protected mode
#=============================#

    cli                   # Disable interrupts
    lgdt [gdt_descriptor] # load GDT

    mov eax, cr0          # Entering protected mode
    or eax, 1             #
    mov cr0, eax          #

    push 0x8              # Flush pipeline with new Segment
    lea eax, [stage2]     #
    push eax              #
    retf                  #


#============#
# DAP Struct
#============#

    dap:                  # Disk Access Packet
        .byte 0x10
        .byte 0
    dap_bcounts:
        .word 1           # Number of blocks to read
    dap_buffer_off:
        .word 0           # Offset of buffer
    dap_buffer_seg:
        .word BUFFER / 16 # Segment of buffer
                          # (Note: Real mode addresses are computed using this formula:
                          # segment * 16 + offset)
    dap_lba:
        .long 1           # Lower 32 bits of starting LBA
        .long 0           # Upper 32 bits of starting LBA

#===========#
# 32bit GDT
#===========#

.code16
GDT:
    gdt_null :
       .quad 0
    gdt_code:
       .word 0xffff        # Limit
       .word 0             # Base
       .byte 0             # Base
       .byte 0b10011010    # 1st flag, Type flag
       .byte 0b11001111    # 2nd flag, Limit
       .byte 0             # Base
    gdt_data:
       .word 0xffff
       .word 0
       .byte 0
       .byte 0b10010010
       .byte 0b11001111
       .byte 0
    gdt_end:

gdt_descriptor :
   .word gdt_end - GDT - 1 # 16-bit size
   .long GDT               # 32-bit start address


#================#
# Boot signature
#================#

    .org 510
    .word 0xAA55 # Boot signature
