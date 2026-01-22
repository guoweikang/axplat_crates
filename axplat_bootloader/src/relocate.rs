//! Kernel relocation support
//!
//! Provides utilities for relocatable kernels that can be loaded at arbitrary
//! physical addresses and properly calculate virtual-to-physical mappings.

use crate::memory::{PhysAddr, VirtAddr};
use crate::traits::RelocatableOps;

/// Default relocatable operations implementation
pub struct DefaultRelocatable;

impl RelocatableOps for DefaultRelocatable {
    fn calculate_phys_virt_offset(link_vaddr: VirtAddr, load_paddr: PhysAddr) -> isize {
        // Calculate the offset between where kernel expects to be (link address)
        // and where it actually is (load address)
        //
        // For a typical kernel:
        // - Linked at: 0xffff_8000_0010_0000 (link_vaddr)
        // - Loaded at: 0x0000_0000_0040_0000 (load_paddr)
        // - Offset = link_vaddr - load_paddr
        link_vaddr.wrapping_sub(load_paddr) as isize
    }

    unsafe fn current_paddr(symbol_vaddr: VirtAddr) -> PhysAddr {
        // For position-independent code, we need to calculate the actual
        // physical address of a symbol based on where we're currently running.
        // This is architecture-specific and typically done in assembly.
        // Here we provide a generic implementation that assumes the symbol
        // address is already a physical address (i.e., running with MMU off).
        symbol_vaddr
    }
}

/// Get current physical address by reading PC-relative position
///
/// This must be implemented differently for each architecture.
///
/// # Safety
/// Must be called before MMU is enabled, with code running at physical addresses.
#[inline(always)]
pub unsafe fn get_current_load_address() -> PhysAddr {
    // This function should be implemented per-architecture
    // For now, we try to read a link-time symbol
    unsafe extern "C" {
        fn _skernel();
    }
    _skernel as *const () as usize
}

/// Calculate the physical-to-virtual offset for linear mapping
///
/// # Arguments
/// * `kernel_link_vaddr` - Virtual address where kernel was linked
/// * `kernel_load_paddr` - Physical address where kernel was loaded
///
/// # Returns
/// The offset that should be added to physical addresses to get virtual addresses
#[inline]
pub fn calculate_linear_map_offset(
    kernel_link_vaddr: VirtAddr,
    kernel_load_paddr: PhysAddr,
) -> usize {
    kernel_link_vaddr.wrapping_sub(kernel_load_paddr)
}

/// Convert virtual address to physical address using the given offset
///
/// # Arguments
/// * `vaddr` - Virtual address
/// * `offset` - Linear mapping offset (vaddr - paddr)
///
/// # Returns
/// Physical address
#[inline]
pub const fn virt_to_phys_with_offset(vaddr: VirtAddr, offset: usize) -> PhysAddr {
    vaddr.wrapping_sub(offset)
}

/// Convert physical address to virtual address using the given offset
///
/// # Arguments
/// * `paddr` - Physical address
/// * `offset` - Linear mapping offset (vaddr - paddr)
///
/// # Returns
/// Virtual address
#[inline]
pub const fn phys_to_virt_with_offset(paddr: PhysAddr, offset: usize) -> VirtAddr {
    paddr.wrapping_add(offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_linear_map_offset() {
        // Typical AArch64 kernel
        let link_vaddr = 0xffff_0000_0010_0000;
        let load_paddr = 0x0000_0000_4000_0000;
        let offset = calculate_linear_map_offset(link_vaddr, load_paddr);

        // offset should be link_vaddr - load_paddr
        assert_eq!(offset, link_vaddr.wrapping_sub(load_paddr));
    }

    #[test]
    fn test_phys_virt_conversion() {
        let offset = 0xffff_0000_0000_0000usize;
        let paddr = 0x4000_0000usize;

        let vaddr = phys_to_virt_with_offset(paddr, offset);
        assert_eq!(vaddr, 0xffff_0000_4000_0000);

        let paddr2 = virt_to_phys_with_offset(vaddr, offset);
        assert_eq!(paddr2, paddr);
    }

    #[test]
    fn test_calculate_phys_virt_offset() {
        let link_vaddr = 0xffff_8000_0010_0000;
        let load_paddr = 0x0000_0000_0040_0000;

        let offset = DefaultRelocatable::calculate_phys_virt_offset(link_vaddr, load_paddr);

        // Verify the offset calculation
        assert_eq!(load_paddr.wrapping_add(offset as usize), link_vaddr);
    }
}
