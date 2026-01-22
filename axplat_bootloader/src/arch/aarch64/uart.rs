//! Simple PL011 UART driver for debugging output

use core::fmt;
use core::ptr;

/// PL011 UART base address on QEMU virt platform
const UART_BASE: usize = 0x09000000;

/// UART registers
const UART_DR: usize = UART_BASE + 0x00;
const UART_FR: usize = UART_BASE + 0x18;

/// Flag register bits
const UART_FR_TXFF: u32 = 1 << 5;

/// Simple UART writer
pub struct Uart;

impl Uart {
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    #[inline]
    pub fn putc(&self, c: u8) {
        unsafe {
            while (ptr::read_volatile(UART_FR as *const u32) & UART_FR_TXFF) != 0 {}
            ptr::write_volatile(UART_DR as *mut u32, c as u32);
        }
    }

    pub fn puts(&self, s: &str) {
        for byte in s.bytes() {
            self.putc(byte);
        }
    }
}

/// Write formatted string to UART
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    struct Writer;
    impl fmt::Write for Writer {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            Uart::new().puts(s);
            Ok(())
        }
    }

    Writer.write_fmt(args).ok();
}
