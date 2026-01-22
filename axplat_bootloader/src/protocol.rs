//! Boot protocol detection (runtime)

use crate::boot_info::BootProtocol;

/// Device Tree Blob magic number (big-endian)
const DTB_MAGIC: u32 = 0xd00dfeed;

/// UEFI System Table signature:  "IBI SYST" (little-endian)
const UEFI_SYSTEM_TABLE_SIGNATURE: u64 = 0x5453595320494249;

/// Detect boot protocol (AArch64)
///
/// # Safety
///
/// Caller must ensure x0 and x1 are the register values passed at boot time
#[cfg(target_arch = "aarch64")]
pub unsafe fn detect_boot_protocol(x0: usize, x1: usize) -> BootProtocol {
    // Method 1: Check if X1 points to a valid UEFI System Table
    if x1 != 0 {
        // First 8 bytes of UEFI System Table is signature
        let potential_signature = unsafe { *(x1 as *const u64) };
        if potential_signature == UEFI_SYSTEM_TABLE_SIGNATURE {
            info!("Detected UEFI boot protocol (System Table at {:#x})", x1);
            return BootProtocol::UEFI;
        }
    }

    // Method 2: Check if X0 points to a valid DTB
    if x0 != 0 {
        // First 4 bytes of DTB is magic (big-endian)
        let potential_magic = unsafe { *(x0 as *const u32) };
        if u32::from_be(potential_magic) == DTB_MAGIC {
            info!("Detected Device Tree boot protocol (DTB at {:#x})", x0);
            return BootProtocol::DeviceTree;
        }
    }

    panic!(
        "Unable to detect boot protocol!  X0={:#x}, X1={:#x}",
        x0, x1
    );
}

/// x86_64 boot protocol detection (future implementation)
#[cfg(target_arch = "x86_64")]
pub unsafe fn detect_boot_protocol(eax: u32, _ebx: usize) -> BootProtocol {
    // Multiboot magic:  0x2BADB002
    const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

    if eax == MULTIBOOT_MAGIC {
        return BootProtocol::Multiboot;
    }

    // TODO:  UEFI detection

    panic!("Unable to detect boot protocol!");
}
