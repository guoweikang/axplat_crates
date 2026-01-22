//! Core traits for bootloader abstraction
//!
//! This module defines the fundamental traits that abstract boot protocol
//! parsing and architecture-specific operations, enabling a clean separation
//! between bootloader and kernel code.

use crate::boot_info::BootInfo;
use crate::memory::{PhysAddr, VirtAddr};

/// Error types for boot protocol parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid magic number or signature
    InvalidMagic,
    /// Corrupted or invalid boot information structure
    InvalidStructure,
    /// Required information is missing
    MissingInformation,
    /// Memory map is invalid or empty
    InvalidMemoryMap,
    /// Unsupported version
    UnsupportedVersion,
}

/// Boot protocol parser trait
///
/// Each boot protocol (Multiboot, Device Tree, UEFI) implements this trait
/// to parse protocol-specific information and convert it to a unified BootInfo.
pub trait BootProtocolParser {
    /// Parse boot information from protocol-specific argument
    ///
    /// # Arguments
    /// * `arg` - Protocol-specific argument (e.g., multiboot info address, DTB address)
    /// * `current_paddr` - Current physical address of running code (for relocation)
    ///
    /// # Returns
    /// Parsed BootInfo or ParseError
    ///
    /// # Safety
    /// - `arg` must point to valid protocol-specific data structure
    /// - `current_paddr` must be the actual physical address where code is loaded
    unsafe fn parse(arg: usize, current_paddr: PhysAddr) -> Result<BootInfo, ParseError>;
}

/// Architecture-specific boot operations
///
/// Each architecture (x86_64, aarch64, riscv64) implements this trait to provide
/// architecture-specific initialization operations needed by the bootloader.
pub trait ArchBootOps {
    /// Initialize boot page table for early kernel mapping
    ///
    /// Creates a minimal page table that maps:
    /// - Identity mapping for bootloader code
    /// - Kernel image at its virtual address
    /// - Linear mapping of physical memory (if applicable)
    ///
    /// # Arguments
    /// * `kernel_paddr` - Physical address where kernel is loaded
    /// * `kernel_vaddr` - Virtual address where kernel expects to run
    /// * `kernel_size` - Size of kernel image in bytes
    ///
    /// # Returns
    /// Physical address of page table root
    ///
    /// # Safety
    /// Must be called before enabling MMU. Page table memory must be properly initialized.
    unsafe fn init_boot_page_table(
        kernel_paddr: PhysAddr,
        kernel_vaddr: VirtAddr,
        kernel_size: usize,
    ) -> PhysAddr;

    /// Enable MMU with the given page table
    ///
    /// # Arguments
    /// * `page_table_root` - Physical address of page table root
    ///
    /// # Safety
    /// - Page table must be properly initialized
    /// - Must be called only once during boot
    /// - Code must be position-independent or properly mapped
    unsafe fn enable_mmu(page_table_root: PhysAddr);

    /// Jump to kernel entry point
    ///
    /// Transfers control to the kernel. This function never returns.
    ///
    /// # Arguments
    /// * `entry` - Virtual address of kernel entry point
    /// * `boot_info` - Pointer to BootInfo structure
    ///
    /// # Safety
    /// - MMU must be enabled with proper mappings
    /// - `entry` must be a valid kernel entry point
    /// - `boot_info` must be accessible from kernel virtual address space
    unsafe fn jump_to_kernel(entry: VirtAddr, boot_info: *const BootInfo) -> !;
}

/// Relocatable address calculation trait
///
/// Provides methods to calculate physical/virtual addresses for relocatable kernels.
pub trait RelocatableOps {
    /// Calculate physical-to-virtual offset based on actual load address
    ///
    /// # Arguments
    /// * `link_vaddr` - Virtual address kernel was linked at
    /// * `load_paddr` - Physical address kernel was actually loaded at
    ///
    /// # Returns
    /// Offset to add to physical addresses to get virtual addresses
    fn calculate_phys_virt_offset(link_vaddr: VirtAddr, load_paddr: PhysAddr) -> isize;

    /// Get current physical address of a symbol
    ///
    /// # Arguments
    /// * `symbol_vaddr` - Virtual address of symbol (from linking)
    ///
    /// # Returns
    /// Current physical address of the symbol
    ///
    /// # Safety
    /// Must be called before MMU is enabled
    unsafe fn current_paddr(symbol_vaddr: VirtAddr) -> PhysAddr;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_size() {
        // Ensure ParseError is small enough to be efficiently passed by value
        assert!(core::mem::size_of::<ParseError>() <= 4);
    }

    #[test]
    fn test_parse_error_debug() {
        let err = ParseError::InvalidMagic;
        // Use Debug trait without format! macro
        let _ = err;
        assert_eq!(err, ParseError::InvalidMagic);
    }
}
