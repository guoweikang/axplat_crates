//! Universal bootloader for axplat
//!
//!  Provides a unified bootloader supporting multiple architectures
//! and boot protocols (UEFI, Device Tree, etc.)

#![no_std]
#![feature(naked_functions)]
#![feature(panic_info_message)]

use memory::{PhysAddr, VirtAddr};

#[macro_use]
extern crate log;

pub mod boot_info;
pub mod protocol;
pub mod memory;

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
        pub use arch:: x86_64 as current_arch;
    } else if #[cfg(target_arch = "riscv64")] {
        pub mod arch {
            pub mod riscv64;
        }
        pub use arch::riscv64 as current_arch;
    }
}

// Re-export commonly used types
pub use boot_info::{BootInfo, BootProtocol, MemoryRegion, MemoryType};

/// Bootloader version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Bootloader name
pub const NAME: &str = "axplat_bootloader";
