//! x86_64 page table setup for bootloader
//!
//! Creates minimal page tables for early kernel boot with:
//! - Identity mapping for bootloader code
//! - Higher-half mapping for kernel
//! - Linear mapping of physical memory

use crate::memory::{PhysAddr, VirtAddr};

/// Page table entry flags
const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const HUGE_PAGE: u64 = 1 << 7;

/// Page table size (4KB, 512 entries)
const PAGE_SIZE: usize = 0x1000;
const ENTRIES_PER_TABLE: usize = 512;

/// Boot page tables (statically allocated)
#[repr(align(4096))]
struct PageTable {
    entries: [u64; ENTRIES_PER_TABLE],
}

impl PageTable {
    const fn new() -> Self {
        Self {
            entries: [0; ENTRIES_PER_TABLE],
        }
    }
}

// Static page tables for boot
static mut BOOT_PML4: PageTable = PageTable::new();
static mut BOOT_PDPT_LOW: PageTable = PageTable::new();
static mut BOOT_PDPT_HIGH: PageTable = PageTable::new();

/// Initialize boot page table
///
/// Creates a page table with:
/// - Identity mapping for low memory (0-512GB)
/// - High mapping for kernel (0xffff_8000_0000_0000 + offset)
///
/// # Returns
/// Physical address of PML4 (page table root)
///
/// # Safety
/// Must be called only once during boot
pub unsafe fn init_boot_page_table(
    _kernel_paddr: PhysAddr,
    _kernel_vaddr: VirtAddr,
    _kernel_size: usize,
) -> PhysAddr {
    // Clear page tables
    unsafe {
        BOOT_PML4 = PageTable::new();
        BOOT_PDPT_LOW = PageTable::new();
        BOOT_PDPT_HIGH = PageTable::new();
    }

    let pml4_paddr = &raw const BOOT_PML4 as usize;
    let pdpt_low_paddr = &raw const BOOT_PDPT_LOW as usize;
    let pdpt_high_paddr = &raw const BOOT_PDPT_HIGH as usize;

    // PML4[0] -> PDPT_LOW (covers 0x0000_0000_0000_0000 - 0x0000_007f_ffff_ffff, 512GB)
    unsafe {
        BOOT_PML4.entries[0] = pdpt_low_paddr as u64 | PRESENT | WRITABLE;
    }

    // PML4[256] -> PDPT_HIGH (covers 0xffff_8000_0000_0000 - 0xffff_807f_ffff_ffff, 512GB)
    unsafe {
        BOOT_PML4.entries[256] = pdpt_high_paddr as u64 | PRESENT | WRITABLE;
    }

    // Identity map first 512GB using 1GB huge pages
    // PDPT_LOW[0..512] -> 1GB pages at 0GB, 1GB, 2GB, ... 511GB
    for i in 0..ENTRIES_PER_TABLE {
        let paddr = (i * 0x4000_0000) as u64; // i * 1GB
        unsafe {
            BOOT_PDPT_LOW.entries[i] = paddr | PRESENT | WRITABLE | HUGE_PAGE;
        }
    }

    // Map high half to same physical memory (linear mapping)
    // PDPT_HIGH[0..512] -> 1GB pages at 0GB, 1GB, 2GB, ... 511GB
    for i in 0..ENTRIES_PER_TABLE {
        let paddr = (i * 0x4000_0000) as u64; // i * 1GB
        unsafe {
            BOOT_PDPT_HIGH.entries[i] = paddr | PRESENT | WRITABLE | HUGE_PAGE;
        }
    }

    pml4_paddr
}

/// Get physical address of boot PML4
pub fn get_boot_page_table() -> PhysAddr {
    &raw const BOOT_PML4 as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table_alignment() {
        // Page tables must be 4KB aligned
        assert_eq!(core::mem::align_of::<PageTable>(), 4096);
    }

    #[test]
    fn test_page_table_size() {
        assert_eq!(core::mem::size_of::<PageTable>(), 4096);
    }

    #[test]
    fn test_page_flags() {
        assert_eq!(PRESENT, 0b001);
        assert_eq!(WRITABLE, 0b010);
        assert_eq!(HUGE_PAGE, 0b10000000);
    }
}
