ENTRY(_start)
OUTPUT_FORMAT(binary)

SECTIONS {
    . = 1M;

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