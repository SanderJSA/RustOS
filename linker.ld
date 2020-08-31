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
    _stage2_sectors = (_stage2_end - _stage2_start) >> 9;

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
    _kernel_sectors = (. - _kernel_start) >> 9;

    /DISCARD/ :
    {
        *(.comment)
    }
}
