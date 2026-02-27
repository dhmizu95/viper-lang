// Build script for Viper compiler
// Links against the C runtime library and GMP

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

    // Link against GMP library for BigInt support
    // Use pkg-config to find GMP if available
    if pkg_config::Config::new().atleast_version("6.0").probe("gmp").is_ok() {
        println!("cargo:rustc-link-lib=gmp");
    } else {
        // Fallback: try to link directly
        println!("cargo:rustc-link-lib=gmp");
    }

    // Rebuild if runtime changes
    println!("cargo:rerun-if-changed=runtime/");
}
