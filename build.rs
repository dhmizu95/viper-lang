// Build script for Viper compiler
// Links against the C runtime library and optionally GMP

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for libraries in the runtime directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let runtime_dir = manifest_dir.join("runtime");

    println!("cargo:rustc-link-search=native={}", runtime_dir.display());

    // Link against the viper runtime library
    // Note: This is optional during initial development
    // println!("cargo:rustc-link-lib=static=viper");

    // Link against GMP library for BigInt support (optional)
    // Try multiple detection methods
    let gmp_found = check_gmp();
    
    if gmp_found {
        println!("cargo:rustc-link-lib=gmp");
        println!("cargo:rustc-cfg=feature=\"bigint\"");
        println!("cargo:warning=GMP found - BigInt support enabled");
    } else {
        println!("cargo:warning=GMP not found - BigInt support disabled");
        println!("cargo:warning=Install libgmp-dev to enable BigInt support");
    }

    // Rebuild if runtime changes
    println!("cargo:rerun-if-changed=runtime/");
}

/// Check if GMP is available via pkg-config or direct library check
fn check_gmp() -> bool {
    // Method 1: Try pkg-config first
    if pkg_config::Config::new()
        .atleast_version("6.0")
        .probe("gmp")
        .is_ok()
    {
        return true;
    }
    
    // Method 2: Check for GMP library files directly
    let gmp_paths = [
        "/usr/lib/x86_64-linux-gnu/libgmp.a",
        "/usr/lib/x86_64-linux-gnu/libgmp.so",
        "/usr/lib/libgmp.a",
        "/usr/lib/libgmp.so",
        "/usr/local/lib/libgmp.a",
        "/usr/local/lib/libgmp.so",
    ];
    
    for path in &gmp_paths {
        if std::path::Path::new(path).exists() {
            return true;
        }
    }
    
    false
}
