//! Boot protocol parsers
//!
//! This module contains parsers for various boot protocols:
//! - Multiboot 1/2 (x86)
//! - Device Tree (ARM, RISC-V)
//! - UEFI (future)

pub mod devicetree;
pub mod multiboot;

pub use devicetree::DeviceTreeParser;
pub use multiboot::MultibootParser;
