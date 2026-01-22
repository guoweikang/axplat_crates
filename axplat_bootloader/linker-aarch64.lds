/* Linker script template for axplat_bootloader on AArch64 */
/* Compatible with StarryOS/arceos linker script structure */

OUTPUT_ARCH(aarch64)
ENTRY(_start)

SECTIONS
{
    /* Link at high virtual address */
    . = 0xffffffff80000000;

    _bootloader_start = .;
    _skernel = .;

    /* Boot code section */
    .text :  ALIGN(4K) {
        _stext = .;
        *(.text.boot)
        *(.text . text.*)
        . = ALIGN(4K);
        _etext = .;
    }

    /* Read-only data */
    _srodata = .;
    . rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
    }

    /* C++ global constructors */
    . init_array : ALIGN(0x10) {
        __init_array_start = .;
        *(.init_array .init_array.*)
        __init_array_end = .;
    }

    /* DWARF debug sections placeholder */
    %DWARF%

    .  = ALIGN(4K);
    _erodata = .;

    /* Data section */
    .data :  ALIGN(4K) {
        _sdata = .;
        *(.data. boot_page_table)
        . = ALIGN(4K);
        *(.data . data.*)
        *(.sdata .sdata.*)
        *(.got . got.*)

        /* Driver registration */
        . = ALIGN(0x10);
        __sdriver_register = .;
        KEEP(*(. driver. register*))
        __edriver_register = .;

        /* Exception table */
        . = ALIGN(0x10);
        _ex_table_start = .;
        KEEP(*(__ex_table))
        _ex_table_end = .;
    }

    /* Thread-local storage data */
    .tdata :  ALIGN(0x10) {
        _stdata = .;
        *(. tdata .tdata.*)
        _etdata = .;
    }

    /* Thread-local storage BSS */
    .tbss :  ALIGN(0x10) {
        _stbss = .;
        *(.tbss .tbss.*)
        *(.tcommon)
        _etbss = . ;
    }

    /* Per-CPU variables */
    .  = ALIGN(4K);
    _percpu_start = .;

    .percpu 0x0 : AT(_percpu_start) {
        _percpu_load_start = .;
        *(.percpu .percpu.*)
        _percpu_load_end = .;
        . = ALIGN(64) * %CPU_NUM%;
    }

    _percpu_size = SIZEOF(.percpu);
    _percpu_end = _percpu_start + (_percpu_size * %CPU_NUM%);
    .  = _percpu_end;

    .  = ALIGN(4K);
    _edata = .;

    /* BSS section */
    .bss : ALIGN(4K) {
        boot_stack = .;
        _boot_stack_start = .;
        *(.bss. stack)
        . = ALIGN(4K);
        boot_stack_top = .;
        _boot_stack_end = .;

        _sbss = .;
        *(. bss . bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
        _ebss = .;
    }

    _bootloader_end = .;
    _ekernel = . ;

    /DISCARD/ : {
        *(. comment)
        *(.gnu*)
        *(.note*)
        *(.eh_frame*)
    }
}

/* Rust linkme support */
SECTIONS {
    linkme_IRQ :  { KEEP(*(linkme_IRQ)) }
    linkm2_IRQ : { KEEP(*(linkm2_IRQ)) }
    linkme_PAGE_FAULT : { KEEP(*(linkme_PAGE_FAULT)) }
    linkm2_PAGE_FAULT : { KEEP(*(linkm2_PAGE_FAULT)) }
    scope_local :  { KEEP(*(scope_local)) }
}
INSERT AFTER .tbss;

/* Exported symbols */
PROVIDE(_bootloader_size = _bootloader_end - _bootloader_start);
PROVIDE(_boot_stack_size = _boot_stack_end - _boot_stack_start);
