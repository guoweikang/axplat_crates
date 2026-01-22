//! x86_64 boot implementation
//!
//! Provides x86_64-specific boot operations for multiboot protocol.

use crate::boot_info::BootInfo;
use crate::memory::{PhysAddr, VirtAddr};
use crate::protocols::MultibootParser;
use crate::traits::{ArchBootOps, BootProtocolParser};

use super::page_table::init_boot_page_table as setup_page_table;

/// x86_64 boot operations
pub struct X86_64BootOps;

impl ArchBootOps for X86_64BootOps {
    unsafe fn init_boot_page_table(
        kernel_paddr: PhysAddr,
        kernel_vaddr: VirtAddr,
        kernel_size: usize,
    ) -> PhysAddr {
        unsafe { setup_page_table(kernel_paddr, kernel_vaddr, kernel_size) }
    }

    unsafe fn enable_mmu(page_table_root: PhysAddr) {
        // Load CR3 with page table root
        unsafe {
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) page_table_root,
                options(nostack, preserves_flags)
            );
        }
    }

    unsafe fn jump_to_kernel(entry: VirtAddr, boot_info: *const BootInfo) -> ! {
        // Prepare arguments according to x86_64 calling convention
        // RDI = first argument (boot_info pointer)
        unsafe {
            core::arch::asm!(
                "mov rdi, {boot_info}",
                "jmp {entry}",
                boot_info = in(reg) boot_info,
                entry = in(reg) entry,
                options(noreturn)
            );
        }
    }
}

/// Rust entry point called from assembly
///
/// # Arguments
/// * `magic` - Multiboot magic number (should be 0x2BADB002)
/// * `mbi_addr` - Physical address of multiboot information structure
///
/// # Safety
/// Called from assembly with proper setup
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_entry(magic: u32, mbi_addr: usize) -> ! {
    // Verify multiboot magic
    if magic != crate::protocols::multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
        panic!("Invalid multiboot magic: {:#x}", magic);
    }

    // Get current physical address for relocation calculation
    let current_paddr = get_kernel_load_address();

    // Parse multiboot information
    let boot_info = match MultibootParser::parse(mbi_addr, current_paddr) {
        Ok(info) => info,
        Err(e) => panic!("Failed to parse multiboot info: {:?}", e),
    };

    // Setup page table
    let page_table_root = unsafe {
        X86_64BootOps::init_boot_page_table(
            boot_info.kernel_phys_base,
            boot_info.kernel_virt_base,
            0x100000, // TODO: Get actual kernel size
        )
    };

    // Enable MMU
    unsafe {
        X86_64BootOps::enable_mmu(page_table_root);
    }

    // Jump to kernel entry point
    // TODO: Determine actual kernel entry point
    let kernel_entry = boot_info.kernel_virt_base;

    unsafe { X86_64BootOps::jump_to_kernel(kernel_entry, &boot_info as *const BootInfo) }
}

/// Get current kernel load physical address
///
/// # Safety
/// Must be called before MMU is enabled
unsafe fn get_kernel_load_address() -> PhysAddr {
    unsafe extern "C" {
        fn _skernel();
    }
    _skernel as *const () as usize
}

/// Panic handler for bootloader
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Simple panic handler - just print and halt
    if let Some(location) = info.location() {
        // In a real implementation, we'd print to serial/screen
        // For now, just halt
        let _ = (location, info.message());
    }

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
