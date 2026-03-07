//! JSON runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare JSON runtime functions
pub fn declare_json_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();

    // vp_json_loads - Parse JSON string into ViperDict
    // Signature: ViperDict* vp_json_loads(const char* json_str)
    let json_loads_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_json_loads", json_loads_type, None);

    // vp_json_dumps - Convert ViperDict to JSON string
    // Signature: char* vp_json_dumps(ViperDict* dict)
    let json_dumps_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_json_dumps", json_dumps_type, None);

    // vp_json_get_error - Get last JSON parse error
    // Signature: const char* vp_json_get_error(void)
    let json_get_error_type = ptr_type.fn_type(&[], false);
    module.add_function("vp_json_get_error", json_get_error_type, None);

    // vp_json_load_file - Load JSON from a file
    // Signature: ViperDict* vp_json_load_file(const char* filename)
    let json_load_file_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_json_load_file", json_load_file_type, None);

    // vp_json_dump_file - Write JSON to a file
    // Signature: int64_t vp_json_dump_file(ViperDict* dict, const char* filename)
    let json_dump_file_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_json_dump_file", json_dump_file_type, None);

    Ok(())
}
