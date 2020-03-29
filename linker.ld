ENTRY(_start)
OUTPUT_FORMAT(binary)

SECTIONS {
    . = 0x100000;

    .boot :
    {
        *(.kernel_start)
    }
    .text :
    {
        *(.text .text.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.data .data.*)
    }
    .bss :
    {
        *(.bss)
    }
}