//! I/O JIT stub registration - Print and Input functions

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_io_stubs(ee: &ExecutionEngine, module: &Module) {
    // Print functions
    register_stubs!(ee, module, [
        "vp_print_i64" => super::super::io::vp_print_i64,
        "vp_print_f64" => super::super::io::vp_print_f64,
        "vp_print_bool" => super::super::io::vp_print_bool,
        "vp_print_str" => super::super::io::vp_print_str_stub,
        "vp_print_newline" => super::super::io::vp_print_newline,
    ]);
}
