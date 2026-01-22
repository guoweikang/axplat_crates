//! Device Tree boot protocol parser
//!
//! Parses Device Tree Blob (DTB/FDT) and extracts memory and boot information.
//! Used on ARM and RISC-V systems.
//!
//! Reference: <https://devicetree.org/>

use crate::boot_info::{BootInfo, BootProtocol, MemoryRegion, MemoryType};
use crate::memory::PhysAddr;
use crate::traits::{BootProtocolParser, ParseError};

/// DTB magic number (big-endian)
const DTB_MAGIC: u32 = 0xd00dfeed;

/// Maximum memory regions we can parse
const MAX_MEMORY_REGIONS: usize = 32;

/// Static storage for memory regions
static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] = [MemoryRegion {
    start: 0,
    size: 0,
    memory_type: MemoryType::Reserved,
}; MAX_MEMORY_REGIONS];

/// Device Tree parser
pub struct DeviceTreeParser;

impl BootProtocolParser for DeviceTreeParser {
    unsafe fn parse(dtb_addr: usize, current_paddr: PhysAddr) -> Result<BootInfo, ParseError> {
        // Validate DTB pointer
        if dtb_addr == 0 {
            return Err(ParseError::InvalidStructure);
        }

        // Verify DTB magic
        let magic = unsafe { core::ptr::read_unaligned(dtb_addr as *const u32) };
        if u32::from_be(magic) != DTB_MAGIC {
            return Err(ParseError::InvalidMagic);
        }

        let mut boot_info = BootInfo::new(BootProtocol::DeviceTree);

        // Store DTB location
        boot_info.dtb_phys_addr = dtb_addr;

        // Parse device tree using fdt crate (when on aarch64 with fdt feature)
        #[cfg(all(target_arch = "aarch64", feature = "fdt"))]
        {
            let region_count = unsafe { parse_fdt_memory(dtb_addr, &mut MEMORY_REGIONS)? };
            boot_info.memory_regions_ptr = unsafe { MEMORY_REGIONS.as_ptr() };
            boot_info.memory_region_count = region_count;
        }

        #[cfg(not(all(target_arch = "aarch64", feature = "fdt")))]
        {
            // Without fdt crate, we need basic memory detection
            // This is a fallback - production code should use proper FDT parsing
            let region_count =
                unsafe { parse_simple_memory(dtb_addr, core::ptr::addr_of_mut!(MEMORY_REGIONS))? };
            boot_info.memory_regions_ptr = unsafe { core::ptr::addr_of!(MEMORY_REGIONS).cast() };
            boot_info.memory_region_count = region_count;
        }

        // Set kernel addresses
        boot_info.kernel_phys_base = current_paddr;

        // For AArch64, typical kernel virtual address uses 0xffff_0000_0000_0000 offset
        boot_info.kernel_virt_base = if cfg!(target_arch = "aarch64") {
            0xffff_0000_0000_0000 + boot_info.kernel_phys_base
        } else if cfg!(target_arch = "riscv64") {
            // RISC-V typically uses 0xffff_ffc0_0000_0000
            0xffff_ffc0_0000_0000 + boot_info.kernel_phys_base
        } else {
            boot_info.kernel_phys_base
        };

        boot_info.linear_map_offset = boot_info.kernel_virt_base - boot_info.kernel_phys_base;

        Ok(boot_info)
    }
}

/// Parse memory from FDT using fdt crate
#[cfg(all(target_arch = "aarch64", feature = "fdt"))]
unsafe fn parse_fdt_memory(
    dtb_addr: usize,
    regions: &mut [MemoryRegion; MAX_MEMORY_REGIONS],
) -> Result<usize, ParseError> {
    use fdt::Fdt;

    // SAFETY: DTB address verified to have correct magic
    let fdt_slice = unsafe {
        // Read totalsize from DTB header (offset 4, big-endian)
        let totalsize_ptr = (dtb_addr + 4) as *const u32;
        let totalsize = u32::from_be(core::ptr::read_unaligned(totalsize_ptr)) as usize;
        core::slice::from_raw_parts(dtb_addr as *const u8, totalsize)
    };

    let fdt = Fdt::new(fdt_slice).map_err(|_| ParseError::InvalidStructure)?;

    let mut count = 0;

    // Parse /memory nodes
    for node in fdt.all_nodes() {
        if node.name.starts_with("memory") {
            // Get reg property which contains base address and size
            if let Some(mut reg) = node.reg() {
                while let Some(region) = reg.next() {
                    if count >= MAX_MEMORY_REGIONS {
                        break;
                    }

                    regions[count] = MemoryRegion {
                        start: region.starting_address as PhysAddr,
                        size: region.size.unwrap_or(0),
                        memory_type: MemoryType::Usable,
                    };
                    count += 1;
                }
            }
        }
    }

    if count == 0 {
        return Err(ParseError::InvalidMemoryMap);
    }

    Ok(count)
}

/// Simple memory parsing fallback (without fdt crate)
///
/// This is a minimal implementation that assumes a simple device tree
/// structure. Production code should use proper FDT parsing.
#[cfg(not(all(target_arch = "aarch64", feature = "fdt")))]
unsafe fn parse_simple_memory(
    _dtb_addr: usize,
    regions: *mut [MemoryRegion; MAX_MEMORY_REGIONS],
) -> Result<usize, ParseError> {
    // TODO: Implement basic FDT parsing without dependencies
    // For now, provide a default memory region for testing

    // This is a placeholder that assumes QEMU virt machine layout
    unsafe {
        (*regions)[0] = MemoryRegion {
            start: 0x4000_0000,
            size: 0x4000_0000, // 1GB
            memory_type: MemoryType::Usable,
        };
    }

    Ok(1)
}

/// Get DTB size from header
///
/// # Safety
/// - `dtb_addr` must point to valid DTB with correct magic
pub unsafe fn get_dtb_size(dtb_addr: usize) -> usize {
    // Totalsize is at offset 4 in DTB header (big-endian)
    let totalsize_ptr = (dtb_addr + 4) as *const u32;
    u32::from_be(unsafe { core::ptr::read_unaligned(totalsize_ptr) }) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtb_magic() {
        assert_eq!(DTB_MAGIC, 0xd00dfeed);
    }

    #[test]
    fn test_dtb_magic_endianness() {
        let magic_be: u32 = 0xd00dfeed;
        let magic_le = u32::from_be(magic_be);
        assert_eq!(magic_le, 0xd00dfeed);
    }
}
