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

    // Use vendored GMP library (bundled in repository)
    let vendor_gmp_dir = manifest_dir.join("vendor/gmp/lib");
    println!("cargo:rustc-link-search=native={}", vendor_gmp_dir.display());

    // Set rpath so the binary finds the vendored library at runtime
    // Multiple rpaths: first looks relative to binary (for installation), then relative to build dir
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/viper/gmp/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../vendor/gmp/lib");

    // Note: Runtime library is linked via JIT stubs for execution
    // For AOT compilation, link with: -L runtime/obj -lviper
    // println!("cargo:rustc-link-lib=static=viper");

    // Link against GMP library for BigInt JIT stubs
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:warning=GMP found - BigInt support enabled");

    // Rebuild if runtime or vendored GMP changes
    println!("cargo:rerun-if-changed=runtime/");
    println!("cargo:rerun-if-changed=vendor/gmp/");
}
