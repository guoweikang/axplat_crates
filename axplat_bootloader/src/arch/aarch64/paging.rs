//! AArch64 page table setup (simple linear mapping)

use crate::memory::{LINEAR_MAP_OFFSET, PAGE_SIZE_1G, PhysAddr, VirtAddr};

/// Set up simple linear mapping page tables
///
/// Mapping:  VA = PA + LINEAR_MAP_OFFSET
/// Uses 1GB huge pages for fast setup
pub fn setup_linear_mapping(max_memory_gb: usize) -> PhysAddr {
    info!(
        "Setting up linear mapping for {} GB of memory",
        max_memory_gb
    );

    // TODO:
    // 1. Allocate page table memory
    // 2. Set up L0/L1/L2 page tables
    // 3. Use 1GB block mapping for entire physical memory
    // 4. Return page table base address

    unimplemented!("Page table setup not yet implemented");
}
