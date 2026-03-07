//! Regular expression runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare regex runtime functions
pub fn declare_re_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();

    // vp_re_compile - Compile a regex pattern
    // Signature: ViperPattern* vp_re_compile(const char* pattern, int64_t flags)
    let re_compile_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_re_compile", re_compile_type, None);

    // vp_re_pattern_free - Free a compiled pattern
    // Signature: void vp_re_pattern_free(ViperPattern* pattern)
    let re_pattern_free_type = context.void_type().fn_type(&[ptr_type.into()], false);
    module.add_function("vp_re_pattern_free", re_pattern_free_type, None);

    // vp_re_match - Match at beginning of string
    // Signature: ViperMatch* vp_re_match(ViperPattern* pattern, const char* string, int64_t pos)
    let re_match_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_re_match", re_match_type, None);

    // vp_re_search - Search anywhere in string
    // Signature: ViperMatch* vp_re_search(ViperPattern* pattern, const char* string, int64_t pos, int64_t endpos)
    let re_search_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into(), i64_type.into()], false);
    module.add_function("vp_re_search", re_search_type, None);

    // vp_re_findall - Find all matches
    // Signature: ViperList* vp_re_findall(ViperPattern* pattern, const char* string)
    let re_findall_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_re_findall", re_findall_type, None);

    // vp_re_split - Split string by pattern
    // Signature: ViperList* vp_re_split(ViperPattern* pattern, const char* string)
    let re_split_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_re_split", re_split_type, None);

    // vp_re_sub - Substitute matches
    // Signature: char* vp_re_sub(ViperPattern* pattern, const char* repl, const char* string, int64_t count)
    let re_sub_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_re_sub", re_sub_type, None);

    // vp_re_fullmatch - Full match check
    // Signature: int64_t vp_re_fullmatch(ViperPattern* pattern, const char* string)
    let re_fullmatch_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_re_fullmatch", re_fullmatch_type, None);

    // vp_re_escape - Escape special characters
    // Signature: char* vp_re_escape(const char* string)
    let re_escape_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_re_escape", re_escape_type, None);

    // vp_re_get_error - Get regex error message
    // Signature: char* vp_re_get_error(int errcode)
    let re_get_error_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_re_get_error", re_get_error_type, None);

    // vp_re_ignorecase - Return IGNORECASE flag value
    // Signature: int64_t vp_re_ignorecase(void)
    let re_flag_type = i64_type.fn_type(&[], false);
    module.add_function("vp_re_ignorecase", re_flag_type, None);

    // vp_re_multiline - Return MULTILINE flag value
    module.add_function("vp_re_multiline", re_flag_type, None);

    // vp_re_dotall - Return DOTALL flag value
    module.add_function("vp_re_dotall", re_flag_type, None);

    // vp_re_verbose - Return VERBOSE flag value
    module.add_function("vp_re_verbose", re_flag_type, None);

    Ok(())
}
