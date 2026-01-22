//! Multiboot 1 protocol parser
//!
//! Parses Multiboot information structure passed by bootloader (GRUB)
//! and converts it to unified BootInfo format.
//!
//! Reference: <https://www.gnu.org/software/grub/manual/multiboot/multiboot.html>

use crate::boot_info::{BootInfo, BootProtocol, MemoryRegion, MemoryType};
use crate::memory::{PhysAddr, VirtAddr};
use crate::traits::{BootProtocolParser, ParseError};

/// Multiboot magic number passed in EAX
pub const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

/// Multiboot information structure flags
const FLAG_MEM: u32 = 1 << 0; // mem_lower and mem_upper are valid
const FLAG_BOOT_DEVICE: u32 = 1 << 1; // boot_device is valid
const FLAG_CMDLINE: u32 = 1 << 2; // cmdline is valid
const FLAG_MODS: u32 = 1 << 3; // mods_count and mods_addr are valid
const FLAG_AOUT_SYMS: u32 = 1 << 4; // Symbol table (a.out format)
const FLAG_ELF_SHDR: u32 = 1 << 5; // Section header table (ELF format)
const FLAG_MEM_MAP: u32 = 1 << 6; // mmap_length and mmap_addr are valid
const FLAG_DRIVES: u32 = 1 << 7; // drives_length and drives_addr are valid
const FLAG_CONFIG: u32 = 1 << 8; // config_table is valid
const FLAG_BOOTLOADER_NAME: u32 = 1 << 9; // boot_loader_name is valid
const FLAG_APM: u32 = 1 << 10; // APM table is valid
const FLAG_VBE: u32 = 1 << 11; // VBE info is valid

/// Multiboot information structure (passed in EBX)
#[repr(C)]
#[derive(Debug)]
struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    syms: [u32; 4],
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,
}

/// Multiboot memory map entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MultibootMmapEntry {
    size: u32,
    base_addr: u64,
    length: u64,
    entry_type: u32,
}

/// Memory map entry types
const MMAP_TYPE_AVAILABLE: u32 = 1;
const MMAP_TYPE_RESERVED: u32 = 2;
const MMAP_TYPE_ACPI_RECLAIMABLE: u32 = 3;
const MMAP_TYPE_ACPI_NVS: u32 = 4;
const MMAP_TYPE_BAD: u32 = 5;

/// Maximum number of memory regions we can store
const MAX_MEMORY_REGIONS: usize = 32;

/// Static storage for memory regions
static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] = [MemoryRegion {
    start: 0,
    size: 0,
    memory_type: MemoryType::Reserved,
}; MAX_MEMORY_REGIONS];

/// Multiboot protocol parser
pub struct MultibootParser;

impl BootProtocolParser for MultibootParser {
    unsafe fn parse(mbi_addr: usize, current_paddr: PhysAddr) -> Result<BootInfo, ParseError> {
        // Validate multiboot info pointer
        if mbi_addr == 0 {
            return Err(ParseError::InvalidStructure);
        }

        // SAFETY: Bootloader guarantees mbi_addr points to valid MultibootInfo
        let mbi = unsafe { &*(mbi_addr as *const MultibootInfo) };

        let mut boot_info = BootInfo::new(BootProtocol::Multiboot);
        let mut region_count = 0;

        // Parse memory map if available
        if mbi.flags & FLAG_MEM_MAP != 0 {
            region_count = unsafe {
                parse_memory_map(
                    mbi.mmap_addr as usize,
                    mbi.mmap_length as usize,
                    core::ptr::addr_of_mut!(MEMORY_REGIONS),
                )?
            };
        } else if mbi.flags & FLAG_MEM != 0 {
            // Fallback to basic memory info
            region_count = unsafe {
                parse_basic_memory(
                    mbi.mem_lower,
                    mbi.mem_upper,
                    core::ptr::addr_of_mut!(MEMORY_REGIONS),
                )?
            };
        } else {
            return Err(ParseError::MissingInformation);
        }

        // Set memory regions pointer
        boot_info.memory_regions_ptr = unsafe { core::ptr::addr_of!(MEMORY_REGIONS).cast() };
        boot_info.memory_region_count = region_count;

        // Parse command line if available
        if mbi.flags & FLAG_CMDLINE != 0 && mbi.cmdline != 0 {
            let cmdline_ptr = mbi.cmdline as *const u8;
            let cmdline_len = unsafe { c_strlen(cmdline_ptr) };
            boot_info.cmdline_ptr = cmdline_ptr;
            boot_info.cmdline_len = cmdline_len;
        }

        // Calculate kernel load addresses
        // For multiboot, kernel is typically loaded at 1MB
        boot_info.kernel_phys_base = current_paddr;

        // For x86_64, typical kernel virtual address
        boot_info.kernel_virt_base = if cfg!(target_arch = "x86_64") {
            0xffff_8000_0000_0000 + boot_info.kernel_phys_base
        } else {
            boot_info.kernel_phys_base
        };

        boot_info.linear_map_offset = boot_info.kernel_virt_base - boot_info.kernel_phys_base;

        Ok(boot_info)
    }
}

/// Parse multiboot memory map
///
/// # Safety
/// - `mmap_addr` must point to valid multiboot memory map
/// - `mmap_length` must be correct length
unsafe fn parse_memory_map(
    mmap_addr: usize,
    mmap_length: usize,
    regions: *mut [MemoryRegion; MAX_MEMORY_REGIONS],
) -> Result<usize, ParseError> {
    let mut count = 0;
    let mut offset = 0;

    while offset < mmap_length && count < MAX_MEMORY_REGIONS {
        // SAFETY: Bootloader guarantees valid memory map
        let entry_ptr = (mmap_addr + offset) as *const MultibootMmapEntry;
        let entry = unsafe { &*entry_ptr };

        // Convert multiboot memory type to our MemoryType
        let memory_type = match entry.entry_type {
            MMAP_TYPE_AVAILABLE => MemoryType::Usable,
            MMAP_TYPE_ACPI_RECLAIMABLE => MemoryType::AcpiReclaimable,
            MMAP_TYPE_ACPI_NVS => MemoryType::AcpiNvs,
            MMAP_TYPE_BAD => MemoryType::BadMemory,
            _ => MemoryType::Reserved,
        };

        unsafe {
            (*regions)[count] = MemoryRegion {
                start: entry.base_addr as PhysAddr,
                size: entry.length as usize,
                memory_type,
            };
        }

        count += 1;
        offset += entry.size as usize + 4; // size field itself is 4 bytes
    }

    if count == 0 {
        return Err(ParseError::InvalidMemoryMap);
    }

    Ok(count)
}

/// Parse basic memory information (fallback)
///
/// # Safety
/// - Only called when memory map is not available
unsafe fn parse_basic_memory(
    mem_lower: u32,
    mem_upper: u32,
    regions: *mut [MemoryRegion; MAX_MEMORY_REGIONS],
) -> Result<usize, ParseError> {
    let mut count = 0;

    // Lower memory (0 to 640KB)
    if mem_lower > 0 {
        unsafe {
            (*regions)[count] = MemoryRegion {
                start: 0,
                size: (mem_lower as usize) * 1024,
                memory_type: MemoryType::Usable,
            };
        }
        count += 1;
    }

    // Upper memory (above 1MB)
    if mem_upper > 0 && count < MAX_MEMORY_REGIONS {
        unsafe {
            (*regions)[count] = MemoryRegion {
                start: 0x100000, // 1MB
                size: (mem_upper as usize) * 1024,
                memory_type: MemoryType::Usable,
            };
        }
        count += 1;
    }

    if count == 0 {
        return Err(ParseError::InvalidMemoryMap);
    }

    Ok(count)
}

/// Calculate C string length
///
/// # Safety
/// - `ptr` must point to valid null-terminated C string
unsafe fn c_strlen(ptr: *const u8) -> usize {
    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
        if len > 4096 {
            // Sanity limit
            break;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiboot_magic() {
        assert_eq!(MULTIBOOT_BOOTLOADER_MAGIC, 0x2BADB002);
    }

    #[test]
    fn test_memory_type_conversion() {
        assert_eq!(MMAP_TYPE_AVAILABLE, 1);
        assert_eq!(MMAP_TYPE_RESERVED, 2);
        assert_eq!(MMAP_TYPE_ACPI_RECLAIMABLE, 3);
    }

    #[test]
    fn test_multiboot_info_size() {
        // Ensure structure size is reasonable
        assert!(core::mem::size_of::<MultibootInfo>() <= 256);
    }

    #[test]
    fn test_mmap_entry_size() {
        // Multiboot spec defines this structure
        assert_eq!(core::mem::size_of::<MultibootMmapEntry>(), 24);
    }
}
