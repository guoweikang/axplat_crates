# Multiboot 1 entry point for x86_64
#
# This file provides the boot entry point for multiboot-compatible bootloaders (GRUB).
# It transitions from 32-bit protected mode to 64-bit long mode.

.section .multiboot
.align 4

# Multiboot header
.long 0x1BADB002                # magic
.long 0x00000003                # flags: page align + memory info
.long -(0x1BADB002 + 0x00000003)  # checksum

.section .text.boot
.code32

.global _start
_start:
    # Disable interrupts
    cli
    
    # Save multiboot arguments
    # EAX = magic (0x2BADB002)
    # EBX = multiboot info structure address
    mov %eax, %edi  # Save magic in EDI
    mov %ebx, %esi  # Save mbi address in ESI
    
    # Load GDT
    lgdt (boot_gdt_desc)
    
    # Set up segment registers
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss
    
    # Enable PAE (Physical Address Extension)
    mov %cr4, %eax
    or $0x20, %eax      # Set PAE bit (bit 5)
    mov %eax, %cr4
    
    # Load PML4 address into CR3
    lea (boot_pml4), %eax
    mov %eax, %cr3
    
    # Enable long mode in EFER MSR
    mov $0xC0000080, %ecx  # EFER MSR number
    rdmsr
    or $0x100, %eax     # Set LME bit (bit 8)
    wrmsr
    
    # Enable paging and protected mode
    mov %cr0, %eax
    or $0x80000001, %eax  # Set PG (bit 31) and PE (bit 0)
    mov %eax, %cr0
    
    # Long jump to 64-bit code segment
    ljmp $0x08, $entry64

.code64
entry64:
    # Clear segment registers (not used in 64-bit mode)
    xor %ax, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss
    
    # Set up stack
    lea (boot_stack_top), %rsp
    
    # Call Rust entry point
    # Arguments are already in RDI (magic) and RSI (mbi_addr)
    call rust_entry
    
    # Should never return
halt:
    hlt
    jmp halt

.section .rodata
.align 16

# GDT for boot
boot_gdt:
    .quad 0x0000000000000000    # Null descriptor
    .quad 0x00AF9A000000FFFF    # 64-bit code segment
    .quad 0x00CF92000000FFFF    # 64-bit data segment
boot_gdt_end:

boot_gdt_desc:
    .word boot_gdt_end - boot_gdt - 1
    .quad boot_gdt

# Temporary page tables
.section .bss
.align 4096

boot_pml4:
    .skip 4096
boot_pdpt:
    .skip 4096
boot_pd:
    .skip 4096

# Boot stack
.align 16
boot_stack:
    .skip 16384  # 16KB stack
boot_stack_top:
