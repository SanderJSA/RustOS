ENTRY(_start)

SECTIONS {
    . = 0x100000;

    .kernel_start :
    {
        *(.kernel_start)
    }

    .rodata :
    {
        *(.rodata)
    }
    .text :
    {
        *(.text)
    }
    .data :
    {
        *(.data)
    }
    .got :
    {
        *(.got)
    }
    .comment :
    {
        *(.comment)
    }
}
