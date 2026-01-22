//! AArch64 boot entry point - Position-independent code

use crate::protocol::detect_boot_protocol;
use crate::{BootInfo, boot_info::BootProtocol};

// Import linker script symbols (compatible with StarryOS)
unsafe extern "C" {
    static _bootloader_start: u8;
    static _bootloader_end: u8;
    static _skernel: u8;
    static _ekernel: u8;
    static boot_stack: u8;
    static boot_stack_top: u8;
    static _stext: u8;
    static _etext: u8;
    static _srodata: u8;
    static _erodata: u8;
    static _sdata: u8;
    static _edata: u8;
    static _sbss: u8;
    static _ebss: u8;
}

macro_rules! println {
    () => {
        $crate::arch::aarch64::uart::_print(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::arch::aarch64::uart::_print(
            format_args!("{}\n", format_args!($($arg)*))
        )
    };
}

/// Boot stack size
const BOOT_STACK_SIZE: usize = 64 * 1024;

/// Boot stack (allocated in .bss.stack section)
#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0; BOOT_STACK_SIZE];

/// Saved X0 register value
static mut SAVED_X0: usize = 0;

/// Saved X1 register value
static mut SAVED_X1: usize = 0;

/// Discovered physical load address
static mut PHYS_LOAD_ADDR: usize = 0;

/// AArch64 boot entry point with Linux Image Header (Position-independent)
///
/// Documentation: <https://docs.kernel.org/arch/arm64/booting.html>
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub unsafe extern "C" fn _start() -> ! {
    const FLAG_LE: usize = 0b0;
    const FLAG_PAGE_SIZE_4K: usize = 0b10;
    const FLAG_ANY_MEM: usize = 0b1000;

    core::arch::naked_asm!(
        // ARM64 Linux Image Header (64 bytes)
        "add     x13, x18, #0x16",     // MZ magic for PE/COFF compatibility
        "b       2f",                   // Branch to actual entry point

        ".quad   0",                    // Image load offset from start of RAM
        ".quad   _ekernel - _start",    // Effective image size
        ".quad   {flags}",              // Kernel flags (LE, 4KB pages, any memory)
        ".quad   0",                    // reserved
        ".quad   0",                    // reserved
        ".quad   0",                    // reserved
        ".ascii  \"ARM\\x64\"",         // Magic number "ARM\x64"
        ".long   0",                    // reserved (PE COFF offset)

        // Actual entry point (after 64-byte header)
        "2:",
        // Save boot parameters
        "mov x19, x0",
        "mov x20, x1",

        // Discover physical load address using PC-relative addressing
        "adr x21, _start",

        // Store physical load address
        "adrp x25, {phys_load_addr}",
        "str x21, [x25, :lo12:{phys_load_addr}]",

        // Set up boot stack (physical address)
        "adrp x26, {boot_stack}",
        "add x26, x26, :lo12:{boot_stack}",
        "mov x27, {boot_stack_size}",
        "add sp, x26, x27",

        // Save boot parameters to static variables
        "adrp x28, {saved_x0}",
        "str x19, [x28, :lo12:{saved_x0}]",
        "adrp x28, {saved_x1}",
        "str x20, [x28, :lo12:{saved_x1}]",

        // Jump to Rust initialization
        "b {rust_init}",

        flags = const FLAG_LE | FLAG_PAGE_SIZE_4K | FLAG_ANY_MEM,
        phys_load_addr = sym PHYS_LOAD_ADDR,
        boot_stack = sym BOOT_STACK,
        boot_stack_size = const BOOT_STACK_SIZE,
        saved_x0 = sym SAVED_X0,
        saved_x1 = sym SAVED_X1,
        rust_init = sym rust_init,
    )
}

/// Rust initialization code
#[unsafe(no_mangle)]
extern "C" fn rust_init() -> ! {
    let phys_load = unsafe { PHYS_LOAD_ADDR };
    let x0 = unsafe { SAVED_X0 };
    let x1 = unsafe { SAVED_X1 };

    println!("axplat_bootloader starting.. .");
    println!("Physical load address: {:#x}", phys_load);
    println!("Boot registers: X0={:#x}, X1={:#x}", x0, x1);

    let protocol = unsafe { detect_boot_protocol(x0, x1) };
    println!("Detected boot protocol: {:?}", protocol);

    let mut boot_info = BootInfo::new(protocol);
    boot_info.kernel_phys_base = phys_load;

    match protocol {
        BootProtocol::DeviceTree => {
            boot_info.dtb_phys_addr = x0;
            super::dtb::handle_devicetree_boot(&mut boot_info, x0);
        }
        BootProtocol::UEFI => {
            boot_info.uefi_system_table = x1;
            super::uefi::handle_uefi_boot(&mut boot_info, x0, x1);
        }
        _ => {
            panic!("Unsupported boot protocol:  {:?}", protocol);
        }
    }

    panic!("Bootloader:  kernel entry not implemented yet!");
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n! !! PANIC !!!");
    println!("{}", info);
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}
