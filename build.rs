// Build script for Viper compiler
// Links against the C runtime library

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for libraries in the runtime/obj directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let runtime_dir = manifest_dir.join("runtime").join("obj");

    println!("cargo:rustc-link-search=native={}", runtime_dir.display());

    // Link against the viper runtime library
    println!("cargo:rustc-link-lib=static=viper");

    // Rebuild if runtime changes
    println!("cargo:rerun-if-changed=runtime/");
}
