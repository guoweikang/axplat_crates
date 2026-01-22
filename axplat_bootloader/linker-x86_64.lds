/* Linker script for x86_64 bootloader */

OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

/* Base address where kernel is linked */
KERNEL_BASE = 0xffff800000000000;

SECTIONS
{
    /* Boot section at 1MB (typical for multiboot) */
    . = 1M;

    .boot :
    {
        KEEP(*(.multiboot))
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
        *(.bss.stack)
    }

    /* Discard sections */
    /DISCARD/ :
    {
        *(.note*)
        *(.comment*)
        *(.eh_frame*)
    }

    /* Kernel boundaries */
    _skernel = 1M;
    _ekernel = .;
}
