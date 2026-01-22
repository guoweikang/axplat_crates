//! Memory address types and utilities

/// Physical address
pub type PhysAddr = usize;

/// Virtual address
pub type VirtAddr = usize;

/// Linear mapping offset (AArch64 default)
pub const LINEAR_MAP_OFFSET: usize = 0xffff_0000_0000_0000;

/// Convert physical address to linear-mapped virtual address
#[inline]
pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr + LINEAR_MAP_OFFSET
}

/// Convert linear-mapped virtual address to physical address
#[inline]
pub const fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    vaddr - LINEAR_MAP_OFFSET
}

/// Page size (4KB)
pub const PAGE_SIZE_4K: usize = 0x1000;

/// Large page size (2MB)
pub const PAGE_SIZE_2M: usize = 0x20_0000;

/// Huge page size (1GB)
pub const PAGE_SIZE_1G: usize = 0x4000_0000;

/// Align down to 4KB page boundary
#[inline]
pub const fn align_down_4k(addr: usize) -> usize {
    addr & !(PAGE_SIZE_4K - 1)
}

/// Align up to 4KB page boundary
#[inline]
pub const fn align_up_4k(addr: usize) -> usize {
    (addr + PAGE_SIZE_4K - 1) & !(PAGE_SIZE_4K - 1)
}
