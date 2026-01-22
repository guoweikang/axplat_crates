//! Build script for axplat_bootloader

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DWARF");
    println!("cargo:rerun-if-env-changed=CPU_NUM");
    
    match arch.as_str() {
        "aarch64" => {
            generate_linker_script_aarch64(&out_dir);
        }
        "x86_64" => {
            println!("cargo:warning=x86_64 linker script not yet implemented");
        }
        "riscv64" => {
            println!("cargo:warning=riscv64 linker script not yet implemented");
        }
        _ => {
            panic!("Unsupported architecture:  {}", arch);
        }
    }
}

fn generate_linker_script_aarch64(out_dir: &str) {
    // Get the manifest directory (the directory containing Cargo.toml)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    
    // Construct the path to the linker script template
    let linker_script_path = PathBuf::from(&manifest_dir)
        .join("linker-aarch64.lds");
    
    println!("cargo:rerun-if-changed={}", linker_script_path.display());
    
    // Read template
    let ld_content = fs::read_to_string(&linker_script_path)
        .unwrap_or_else(|e| panic!(
            "Failed to read linker script template at {}: {}",
            linker_script_path.display(),
            e
        ));
    
    // Get CPU_NUM from environment, default to 1
    let cpu_num: usize = env::var("CPU_NUM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    
    println!("cargo:warning=Building with CPU_NUM = {}", cpu_num);
    
    // Replace %CPU_NUM% placeholder
    let ld_content = ld_content.replace("%CPU_NUM%", &cpu_num.to_string());
    
    // Replace %DWARF% placeholder
    let dwarf_sections = if env::var("DWARF").is_ok_and(|v| v == "y") {
        r#"debug_abbrev :  { .  += SIZEOF(. debug_abbrev); }
    debug_addr : { . += SIZEOF(.debug_addr); }
    debug_aranges : { . += SIZEOF(.debug_aranges); }
    debug_info : { . += SIZEOF(.debug_info); }
    debug_line : { . += SIZEOF(. debug_line); }
    debug_line_str : { . += SIZEOF(.debug_line_str); }
    debug_ranges : { . += SIZEOF(.debug_ranges); }
    debug_rnglists : { . += SIZEOF(.debug_rnglists); }
    debug_str : { . += SIZEOF(.debug_str); }
    debug_str_offsets : { . += SIZEOF(.debug_str_offsets); }"#
    } else {
        ""
    };
    
    let ld_content = ld_content.replace("%DWARF%", dwarf_sections);
    
    // Write processed linker script to output directory
    let dst = Path::new(out_dir).join("linker.lds");
    fs::write(&dst, ld_content)
        .unwrap_or_else(|e| panic! ("Failed to write linker script:  {}", e));
    
    // Tell cargo to use this linker script
    println!("cargo:rustc-link-arg=-T{}", dst.display());
    
    println!("cargo:warning=Using linker script: {}", dst. display());
}
