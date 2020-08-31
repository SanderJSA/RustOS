ENTRY(bootloader_start)

SECTIONS {
    . = 0x7C00;
    _stack_end = .;

    .stage1 :
    {
        *(.stage1)
    }

    _stage2_start = .;
    .stage2 :
    {
        *(.stage2)
    }
    _stage2_end = .;
    _stage2_size = _stage2_end - _stage2_start;

    . = 0x100000;

    _kernel_start = .;
    .text :
    {
        *(.text)
    }
    .rodata :
    {
        *(.rodata)
    }
    .data.rel.ro :
    {
        *(.data.rel.ro)
    }
    .got :
    {
        *(.got)
    }
    .data :
    {
        *(.data)
    }
    . = ALIGN(512);
    _kernel_size = . - _kernel_start;

    /DISCARD/ :
    {
        *(.comment)
    }
}
