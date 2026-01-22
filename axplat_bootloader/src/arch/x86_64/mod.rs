//! x86_64 architecture support for bootloader
//!
//! Implements x86_64-specific boot operations including:
//! - Multiboot entry point
//! - Page table setup
//! - MMU initialization
//! - Kernel entry

pub mod boot;
pub mod page_table;

pub use boot::*;
pub use page_table::*;
