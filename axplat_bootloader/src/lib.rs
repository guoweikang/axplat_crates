//! Universal bootloader for axplat
//!
//! Provides a unified bootloader supporting multiple architectures
//! and boot protocols (Multiboot, Device Tree, UEFI).
//!
//! # Design Principles
//!
//! 1. **Progressive Integration**: New bootloader can be optionally enabled via feature flags
//! 2. **Relocatable Kernel Support**: Handles kernels loaded at arbitrary physical addresses
//! 3. **Clear Separation**: Bootloader parses protocols → Kernel consumes BootInfo
//! 4. **Multi-arch/Multi-protocol**: Uses traits for abstraction
//!
//! # Architecture
//!
//! ```text
//! Boot Protocol (Multiboot/DT/UEFI)
//!          ↓
//!    BootProtocolParser trait
//!          ↓
//!      BootInfo (unified)
//!          ↓
//!   ArchBootOps trait
//!          ↓
//!    Kernel Entry
//! ```

#![no_std]

extern crate log;

pub mod boot_info;
pub mod memory;
pub mod protocol;
pub mod protocols;
pub mod relocate;
pub mod traits;

// Architecture-specific modules
cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        pub mod arch {
            #[macro_use]
            pub mod aarch64;
        }
        pub use arch::aarch64 as current_arch;
    } else if #[cfg(target_arch = "x86_64")] {
        pub mod arch {
            pub mod x86_64;
        }
        pub use arch::x86_64 as current_arch;
    } else if #[cfg(target_arch = "riscv64")] {
        pub mod arch {
            pub mod riscv64;
        }
        pub use arch::riscv64 as current_arch;
    }
}

// Re-export commonly used types
pub use boot_info::{BootInfo, BootProtocol, DtbInfo, MemoryRegion, MemoryRegions, MemoryType};
pub use protocols::{DeviceTreeParser, MultibootParser};
pub use traits::{ArchBootOps, BootProtocolParser, ParseError, RelocatableOps};

/// Bootloader version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Bootloader name
pub const NAME: &str = "axplat_bootloader";
