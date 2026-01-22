# axplat_bootloader

Universal bootloader for StarryOS supporting multiple architectures and boot protocols.

## Features

- ✅ **Multi-Architecture Support**
  - x86_64 (Multiboot 1)
  - AArch64 (Device Tree, UEFI)
  - RISC-V 64 (Device Tree) - Placeholder
  - LoongArch64 - Planned

- ✅ **Multi-Protocol Support**
  - Multiboot 1 (x86)
  - Device Tree Boot (ARM, RISC-V)
  - UEFI - Planned

- ✅ **Relocatable Kernel**
  - Kernels can be loaded at arbitrary physical addresses
  - Dynamic physical-virtual offset calculation
  - Position-independent boot code

- ✅ **Clean Abstraction**
  - Protocol-independent `BootInfo` structure
  - Trait-based architecture operations
  - Minimal kernel-bootloader coupling

## Design Principles

### 1. Progressive Integration

The bootloader can be optionally integrated via feature flags without breaking existing boot paths:

```toml
[features]
use-bootloader = ["axplat_bootloader"]
```

### 2. Relocatable Kernel Support

The bootloader handles kernels loaded at any physical address:

```rust
// Bootloader calculates actual load address
let current_paddr = get_current_load_address();

// Creates BootInfo with correct offsets
let boot_info = parser.parse(mbi_addr, current_paddr)?;
```

### 3. Clear Separation of Concerns

```text
┌─────────────────────────────────────┐
│  Boot Protocol (Multiboot/DT/UEFI) │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   BootProtocolParser trait          │
│   - parse() → BootInfo              │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   Unified BootInfo Structure        │
│   - memory_regions                  │
│   - kernel_phys_base / virt_base    │
│   - cmdline, dtb, etc.              │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   ArchBootOps trait                 │
│   - init_boot_page_table()          │
│   - enable_mmu()                    │
│   - jump_to_kernel()                │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│        Kernel Entry Point           │
└─────────────────────────────────────┘
```

## Architecture

### Core Types

#### BootInfo
Unified boot information passed from bootloader to kernel:

```rust
#[repr(C)]
pub struct BootInfo {
    pub magic: u32,                    // "AXBT" (0x54425841)
    pub version: u32,
    pub boot_protocol: BootProtocol,
    pub kernel_phys_base: PhysAddr,
    pub kernel_virt_base: VirtAddr,
    pub linear_map_offset: usize,
    pub memory_region_count: usize,
    pub memory_regions_ptr: *const MemoryRegion,
    pub dtb_phys_addr: usize,
    pub cmdline_ptr: *const u8,
    pub cmdline_len: usize,
}
```

#### MemoryRegion
Physical memory descriptor:

```rust
#[repr(C)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub size: usize,
    pub memory_type: MemoryType,  // Usable, Reserved, Kernel, etc.
}
```

### Core Traits

#### BootProtocolParser
Parses protocol-specific information:

```rust
pub trait BootProtocolParser {
    unsafe fn parse(arg: usize, current_paddr: PhysAddr) 
        -> Result<BootInfo, ParseError>;
}
```

**Implementations:**
- `MultibootParser` - Parses Multiboot 1 information
- `DeviceTreeParser` - Parses Device Tree Blob (FDT)

#### ArchBootOps
Architecture-specific boot operations:

```rust
pub trait ArchBootOps {
    unsafe fn init_boot_page_table(
        kernel_paddr: PhysAddr,
        kernel_vaddr: VirtAddr,
        kernel_size: usize,
    ) -> PhysAddr;
    
    unsafe fn enable_mmu(page_table_root: PhysAddr);
    
    unsafe fn jump_to_kernel(
        entry: VirtAddr, 
        boot_info: *const BootInfo
    ) -> !;
}
```

**Implementations:**
- `X86_64BootOps` - x86_64 operations
- `Riscv64BootOps` - RISC-V operations (placeholder)

## Usage

### As a Library

```rust
use axplat_bootloader::{
    BootInfo, MultibootParser, BootProtocolParser,
    X86_64BootOps, ArchBootOps,
};

// In bootloader code
unsafe {
    // Parse multiboot info
    let boot_info = MultibootParser::parse(mbi_addr, current_paddr)?;
    
    // Setup page table
    let pt_root = X86_64BootOps::init_boot_page_table(
        boot_info.kernel_phys_base,
        boot_info.kernel_virt_base,
        kernel_size,
    );
    
    // Enable MMU
    X86_64BootOps::enable_mmu(pt_root);
    
    // Jump to kernel
    X86_64BootOps::jump_to_kernel(kernel_entry, &boot_info);
}
```

### In Kernel

```rust
use axplat_bootloader::BootInfo;

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &BootInfo) {
    // Verify boot info
    assert!(boot_info.is_valid());
    
    // Get usable memory
    for region in boot_info.usable_regions() {
        println!("RAM: {:#x} - {:#x} ({} MB)",
            region.start,
            region.end(),
            region.size / 1024 / 1024
        );
    }
    
    // Get command line
    if let Some(cmdline) = boot_info.cmdline() {
        println!("Command line: {}", cmdline);
    }
}
```

## Implementation Status

### x86_64
- ✅ Multiboot 1 parsing
- ✅ Memory map extraction
- ✅ Page table setup (PML4 with 1GB pages)
- ✅ MMU enable
- ✅ Kernel entry
- ⚠️ TODO: Multiboot 2 support
- ⚠️ TODO: Module parsing

### AArch64
- ✅ Device Tree detection
- ✅ Entry point (_start)
- ⚠️ TODO: Complete FDT memory parsing
- ⚠️ TODO: Page table setup
- ⚠️ TODO: UEFI support

### RISC-V 64
- ⚠️ Placeholder only
- TODO: OpenSBI integration
- TODO: Device Tree parsing
- TODO: SV39/SV48 page tables

### LoongArch64
- ❌ Not started
- TODO: All functionality

## Building

### Prerequisites

```bash
# Install Rust nightly
rustup toolchain install nightly

# Add targets
rustup target add x86_64-unknown-none
rustup target add aarch64-unknown-none
rustup target add riscv64gc-unknown-none-elf
```

### Build for Specific Architecture

```bash
# x86_64
cargo build --target x86_64-unknown-none

# AArch64
cargo build --target aarch64-unknown-none

# RISC-V
cargo build --target riscv64gc-unknown-none-elf
```

### Run Tests

```bash
cargo test
```

### Check Code Quality

```bash
# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt --check
```

## Testing

### Unit Tests

```bash
cargo test --lib
```

Tests cover:
- BootInfo structure validation
- Memory region operations
- Multiboot parsing (with mock data)
- Address calculations

### Integration Tests

(TODO: Requires QEMU environment)

```bash
# x86_64 QEMU test
make test-x86_64

# AArch64 QEMU test
make test-aarch64
```

## Project Structure

```
axplat_bootloader/
├── Cargo.toml              # Crate metadata and dependencies
├── build.rs                # Build script for linker script generation
├── README.md               # This file
│
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── boot_info.rs        # BootInfo and related types
│   ├── traits.rs           # Core trait definitions
│   ├── relocate.rs         # Relocation utilities
│   ├── protocol.rs         # Runtime protocol detection
│   ├── memory.rs           # Memory types and utilities
│   │
│   ├── protocols/          # Boot protocol parsers
│   │   ├── mod.rs
│   │   ├── multiboot.rs    # Multiboot 1/2 parser
│   │   └── devicetree.rs   # Device Tree parser
│   │
│   └── arch/               # Architecture-specific implementations
│       ├── x86_64/
│       │   ├── mod.rs
│       │   ├── boot.rs     # Rust boot logic
│       │   ├── multiboot.S # Assembly entry point
│       │   └── page_table.rs
│       ├── aarch64/
│       │   ├── mod.rs
│       │   ├── entry.rs
│       │   ├── dtb.rs
│       │   └── paging.rs
│       └── riscv64/
│           └── ... (placeholder)
│
├── linker-x86_64.lds       # x86_64 linker script
├── linker-aarch64.lds      # AArch64 linker script
└── linker-riscv64.lds      # RISC-V linker script
```

## Contributing

### Adding a New Architecture

1. Create `src/arch/<arch>/` directory
2. Implement `ArchBootOps` trait
3. Add linker script `linker-<arch>.lds`
4. Update `build.rs` to handle new architecture
5. Add tests

### Adding a New Boot Protocol

1. Create `src/protocols/<protocol>.rs`
2. Implement `BootProtocolParser` trait
3. Add protocol detection logic
4. Add tests with mock data

## References

- [Multiboot Specification](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html)
- [Device Tree Specification](https://devicetree.org/)
- [rust-osdev/bootloader](https://github.com/rust-osdev/bootloader)
- [hermit-os/loader](https://github.com/hermit-os/loader)

## License

Apache-2.0

## Authors

See workspace Cargo.toml for full list of contributors.
