// Build script for Viper compiler
// Links against GMP for BigInt support via C bridge

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for libraries in the runtime/obj directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let runtime_dir = manifest_dir.join("runtime").join("obj");

    println!("cargo:rustc-link-search=native={}", runtime_dir.display());
    println!("cargo:rustc-link-search=native={}/obj", runtime_dir.display());

    // Link against Viper runtime library (for non-BigInt runtime functions)
    println!("cargo:rustc-link-lib=static=viper");

    // Link against GMP for BigInt support
    println!("cargo:rustc-link-lib=gmp");

    // Rebuild if runtime changes
    println!("cargo:rerun-if-changed=runtime/");
}
