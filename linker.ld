ENTRY(k_start)
STARTUP(build/kernel_start.o)

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
