// Build script for Viper compiler
// Links against GMP for BigInt JIT stubs

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for libraries in the runtime directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let runtime_dir = manifest_dir.join("runtime");

    println!("cargo:rustc-link-search=native={}", runtime_dir.display());
    println!("cargo:rustc-link-search=native={}/obj", runtime_dir.display());
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");

    // Note: Runtime library is linked via JIT stubs for execution
    // For AOT compilation, link with: -L runtime/obj -lviper
    // println!("cargo:rustc-link-lib=static=viper");

    // Link against GMP library for BigInt JIT stubs
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:warning=GMP found - BigInt support enabled");

    // Rebuild if runtime changes
    println!("cargo:rerun-if-changed=runtime/");
}
