/* Linker script for RISC-V 64-bit bootloader */

OUTPUT_ARCH(riscv)
ENTRY(_start)

/* Base physical address (QEMU virt default) */
BASE_ADDRESS = 0x80000000;

SECTIONS
{
    . = BASE_ADDRESS;

    _skernel = .;

    /* Boot section */
    .text.boot : ALIGN(4K)
    {
        KEEP(*(.text.boot))
    }

    /* Main code */
    .text : ALIGN(4K)
    {
        _stext = .;
        *(.text .text.*)
        _etext = .;
    }

    /* Read-only data */
    .rodata : ALIGN(4K)
    {
        _srodata = .;
        *(.rodata .rodata.*)
        _erodata = .;
    }

    /* Data */
    .data : ALIGN(4K)
    {
        _sdata = .;
        *(.data .data.*)
        _edata = .;
    }

    /* BSS */
    .bss : ALIGN(4K)
    {
        _sbss = .;
        *(.bss .bss.*)
        *(COMMON)
        _ebss = .;
    }

    /* Stack */
    .bss.stack (NOLOAD) : ALIGN(16)
    {
        _sstack = .;
        . += 0x10000; /* 64KB stack */
        _estack = .;
    }

    /* Discard sections */
    /DISCARD/ :
    {
        *(.note*)
        *(.comment*)
        *(.eh_frame*)
    }

    _ekernel = .;
}
