//! RISC-V 64-bit architecture support (placeholder)
//!
//! TODO: Implement RISC-V boot protocol and page table setup
//!
//! RISC-V typically boots via:
//! - OpenSBI (Supervisor Binary Interface)
//! - Device Tree in A1 register
//! - Hart ID in A0 register

// Placeholder module - to be implemented

pub mod boot;

pub use boot::*;
