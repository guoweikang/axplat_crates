//! Boot information passed from bootloader to kernel

use crate::memory::{PhysAddr, VirtAddr};

/// Boot protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BootProtocol {
    /// Device Tree boot (X0 = DTB address)
    DeviceTree = 1,
    /// UEFI boot (X0 = ImageHandle, X1 = SystemTable)
    UEFI = 2,
    /// Multiboot boot (x86)
    Multiboot = 3,
    /// BIOS boot (x86)
    BIOS = 4,
}

/// Physical memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    /// Usable memory
    Usable = 1,
    /// Reserved memory
    Reserved = 2,
    /// ACPI reclaimable memory
    AcpiReclaimable = 3,
    /// ACPI NVS memory
    AcpiNvs = 4,
    /// Bad memory
    BadMemory = 5,
    /// Memory used by bootloader
    BootloaderReserved = 6,
    /// Memory occupied by kernel image
    Kernel = 7,
}

/// Physical memory region
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion {
    /// Start physical address
    pub start: PhysAddr,
    /// Size in bytes
    pub size: usize,
    /// Memory type
    pub memory_type: MemoryType,
}

impl MemoryRegion {
    pub const fn new(start: PhysAddr, size: usize, memory_type: MemoryType) -> Self {
        Self {
            start,
            size,
            memory_type,
        }
    }

    /// End address (exclusive)
    pub const fn end(&self) -> PhysAddr {
        self.start + self.size
    }

    /// Check if the region contains the specified address
    pub const fn contains(&self, addr: PhysAddr) -> bool {
        addr >= self.start && addr < self.end()
    }
}

/// Boot information passed from bootloader to kernel
#[repr(C)]
pub struct BootInfo {
    /// Magic number:  "AXBT" (0x54425841)
    pub magic: u32,

    /// BootInfo structure version
    pub version: u32,

    /// Boot protocol
    pub boot_protocol: BootProtocol,

    /// Physical address where kernel is loaded
    pub kernel_phys_base: PhysAddr,

    /// Virtual address where kernel is currently running
    pub kernel_virt_base: VirtAddr,

    /// Linear mapping offset (VA = PA + offset)
    pub linear_map_offset: usize,

    /// Number of physical memory regions
    pub memory_region_count: usize,

    /// Pointer to physical memory region array
    pub memory_regions_ptr: *const MemoryRegion,

    /// Device Tree Blob physical address (if available)
    pub dtb_phys_addr: usize,

    /// UEFI System Table physical address (if available)
    pub uefi_system_table: usize,

    /// Command line arguments pointer (C string)
    pub cmdline_ptr: *const u8,

    /// Command line arguments length
    pub cmdline_len: usize,
}

impl BootInfo {
    /// Magic number: "AXBT"
    pub const MAGIC: u32 = 0x54425841;

    /// Current version number
    pub const VERSION: u32 = 1;

    /// Create empty BootInfo
    pub const fn new(boot_protocol: BootProtocol) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            boot_protocol,
            kernel_phys_base: 0,
            kernel_virt_base: 0,
            linear_map_offset: 0,
            memory_region_count: 0,
            memory_regions_ptr: core::ptr::null(),
            dtb_phys_addr: 0,
            uefi_system_table: 0,
            cmdline_ptr: core::ptr::null(),
            cmdline_len: 0,
        }
    }

    /// Get memory region list
    pub fn memory_regions(&self) -> &[MemoryRegion] {
        if self.memory_regions_ptr.is_null() {
            &[]
        } else {
            unsafe {
                core::slice::from_raw_parts(self.memory_regions_ptr, self.memory_region_count)
            }
        }
    }

    /// Get command line arguments
    pub fn cmdline(&self) -> Option<&str> {
        if self.cmdline_ptr.is_null() || self.cmdline_len == 0 {
            None
        } else {
            unsafe {
                let slice = core::slice::from_raw_parts(self.cmdline_ptr, self.cmdline_len);
                core::str::from_utf8(slice).ok()
            }
        }
    }

    /// Verify magic and version
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC && self.version == Self::VERSION
    }

    /// Find usable memory regions
    pub fn usable_regions(&self) -> impl Iterator<Item = &MemoryRegion> {
        self.memory_regions()
            .iter()
            .filter(|r| r.memory_type == MemoryType::Usable)
    }

    /// Calculate total usable memory size
    pub fn total_usable_memory(&self) -> usize {
        self.usable_regions().map(|r| r.size).sum()
    }
}

/// Memory regions collection with helper methods
pub struct MemoryRegions {
    regions: &'static [MemoryRegion],
}

impl MemoryRegions {
    /// Create new MemoryRegions from slice
    pub const fn new(regions: &'static [MemoryRegion]) -> Self {
        Self { regions }
    }

    /// Get all regions
    pub const fn as_slice(&self) -> &'static [MemoryRegion] {
        self.regions
    }

    /// Get iterator over regions
    pub fn iter(&self) -> core::slice::Iter<'static, MemoryRegion> {
        self.regions.iter()
    }

    /// Find region containing the given address
    pub fn find_region(&self, addr: PhysAddr) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    /// Get total size of all usable memory
    pub fn total_usable(&self) -> usize {
        self.regions
            .iter()
            .filter(|r| r.memory_type == MemoryType::Usable)
            .map(|r| r.size)
            .sum()
    }
}

impl<'a> IntoIterator for &'a MemoryRegions {
    type Item = &'a MemoryRegion;
    type IntoIter = core::slice::Iter<'a, MemoryRegion>;

    fn into_iter(self) -> Self::IntoIter {
        self.regions.iter()
    }
}

/// Device tree blob information
#[derive(Debug, Clone, Copy)]
pub struct DtbInfo {
    /// Physical address of DTB
    pub phys_addr: PhysAddr,
    /// Size of DTB in bytes
    pub size: usize,
}

impl DtbInfo {
    /// Create new DtbInfo
    pub const fn new(phys_addr: PhysAddr, size: usize) -> Self {
        Self { phys_addr, size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_info_magic() {
        assert_eq!(BootInfo::MAGIC, 0x54425841); // "AXBT" in ASCII
    }

    #[test]
    fn test_boot_info_new() {
        let info = BootInfo::new(BootProtocol::Multiboot);
        assert_eq!(info.magic, BootInfo::MAGIC);
        assert_eq!(info.version, BootInfo::VERSION);
        assert_eq!(info.boot_protocol, BootProtocol::Multiboot);
        assert!(info.is_valid());
    }

    #[test]
    fn test_memory_region() {
        let region = MemoryRegion::new(0x1000, 0x10000, MemoryType::Usable);
        assert_eq!(region.start, 0x1000);
        assert_eq!(region.size, 0x10000);
        assert_eq!(region.end(), 0x11000);
        assert!(region.contains(0x5000));
        assert!(!region.contains(0x500));
        assert!(!region.contains(0x11000));
    }

    #[test]
    fn test_memory_regions() {
        static REGIONS_DATA: [MemoryRegion; 3] = [
            MemoryRegion {
                start: 0x1000,
                size: 0x10000,
                memory_type: MemoryType::Usable,
            },
            MemoryRegion {
                start: 0x20000,
                size: 0x10000,
                memory_type: MemoryType::Reserved,
            },
            MemoryRegion {
                start: 0x40000,
                size: 0x20000,
                memory_type: MemoryType::Usable,
            },
        ];
        let regions = MemoryRegions::new(&REGIONS_DATA);

        assert_eq!(regions.as_slice().len(), 3);
        assert_eq!(regions.total_usable(), 0x30000);

        let found = regions.find_region(0x5000);
        assert!(found.is_some());
        assert_eq!(found.unwrap().memory_type, MemoryType::Usable);
    }

    #[test]
    fn test_boot_info_usable_memory() {
        static REGIONS: [MemoryRegion; 3] = [
            MemoryRegion {
                start: 0x1000,
                size: 0x1000,
                memory_type: MemoryType::Usable,
            },
            MemoryRegion {
                start: 0x3000,
                size: 0x1000,
                memory_type: MemoryType::Reserved,
            },
            MemoryRegion {
                start: 0x5000,
                size: 0x2000,
                memory_type: MemoryType::Usable,
            },
        ];

        let mut info = BootInfo::new(BootProtocol::DeviceTree);
        info.memory_regions_ptr = REGIONS.as_ptr();
        info.memory_region_count = REGIONS.len();

        assert_eq!(info.total_usable_memory(), 0x3000);
        assert_eq!(info.usable_regions().count(), 2);
    }

    #[test]
    fn test_dtb_info() {
        let dtb = DtbInfo::new(0x40000000, 0x10000);
        assert_eq!(dtb.phys_addr, 0x40000000);
        assert_eq!(dtb.size, 0x10000);
    }
}
