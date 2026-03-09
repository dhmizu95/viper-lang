//! Strings JIT stub registration - String and Format functions

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_string_stubs(ee: &ExecutionEngine, module: &Module) {
    // String functions
    register_stubs!(ee, module, [
        "vp_str_concat" => super::super::strings::vp_str_concat_stub,
        "vp_str_repeat" => super::super::strings::vp_str_repeat_stub,
        "vp_str_from_i64" => super::super::strings::vp_str_from_i64_stub,
        "vp_str_from_f64" => super::super::strings::vp_str_from_f64_stub,
        "vp_str_len" => super::super::strings::vp_str_len_stub,
        "vp_str_create" => super::super::strings::vp_str_create_stub,
        "vp_str_upper" => super::super::strings::vp_str_upper_stub,
        "vp_str_lower" => super::super::strings::vp_str_lower_stub,
        "vp_str_split" => super::super::strings::vp_str_split_stub,
        "vp_str_replace" => super::super::strings::vp_str_replace_stub,
        "vp_str_from_bool" => super::super::strings::vp_str_from_bool_stub,
        "vp_str_format" => super::super::strings::vp_str_format_stub,
        "vp_str_equals" => super::super::strings::vp_str_equals_stub,
        "vp_str_compare" => super::super::strings::vp_str_compare_stub,
        "vp_exit" => super::super::strings::vp_exit_stub,
    ]);

    // Bytes functions
    register_stubs!(ee, module, [
        "vp_bytes_create" => super::super::strings::vp_bytes_create_stub,
        "vp_bytes_free" => super::super::strings::vp_bytes_free_stub,
        "vp_bytes_len" => super::super::strings::vp_bytes_len_stub,
        "vp_bytes_get" => super::super::strings::vp_bytes_get_stub,
        "vp_bytes_print" => super::super::strings::vp_bytes_print_stub,
    ]);
}
