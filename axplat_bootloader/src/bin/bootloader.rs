//! Standalone bootloader binary

#![no_std]
#![no_main]

// The bootloader entry point is already defined in the architecture-specific modules
// This file just exists to create a binary target

use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
