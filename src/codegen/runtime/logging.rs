//! Logging runtime function declarations for Viper code generation

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare logging runtime functions
pub fn declare_logging_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();
    let void_type = context.void_type();

    // vp_logging_create_logger - Create a new logger
    // Signature: ViperLogger* vp_logging_create_logger(const char* name, int64_t level)
    let create_logger_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_logging_create_logger", create_logger_type, None);

    // vp_logging_logger_free - Free a logger
    // Signature: void vp_logging_logger_free(ViperLogger* logger)
    let logger_free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_logging_logger_free", logger_free_type, None);

    // vp_logging_set_level - Set logger level
    // Signature: void vp_logging_set_level(ViperLogger* logger, int64_t level)
    let set_level_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_logging_set_level", set_level_type, None);

    // vp_logging_get_level - Get logger level
    // Signature: int64_t vp_logging_get_level(ViperLogger* logger)
    let get_level_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_logging_get_level", get_level_type, None);

    // vp_logging_debug - Log debug message
    // Signature: void vp_logging_debug(ViperLogger* logger, const char* msg)
    let log_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_logging_debug", log_type, None);

    // vp_logging_info - Log info message
    module.add_function("vp_logging_info", log_type, None);

    // vp_logging_warning - Log warning message
    module.add_function("vp_logging_warning", log_type, None);

    // vp_logging_error - Log error message
    module.add_function("vp_logging_error", log_type, None);

    // vp_logging_critical - Log critical message
    module.add_function("vp_logging_critical", log_type, None);

    // vp_logging_exception - Log exception message
    module.add_function("vp_logging_exception", log_type, None);

    // vp_logging_set_format - Set log format string
    // Signature: void vp_logging_set_format(ViperLogger* logger, const char* format)
    let set_format_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_logging_set_format", set_format_type, None);

    // vp_logging_add_handler - Add file handler to logger
    // Signature: int64_t vp_logging_add_handler(ViperLogger* logger, const char* filepath)
    let add_handler_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_logging_add_handler", add_handler_type, None);

    // Level constants
    // vp_logging_debug_level - Return DEBUG level value (0)
    let level_type = i64_type.fn_type(&[], false);
    module.add_function("vp_logging_debug_level", level_type, None);

    // vp_logging_info_level - Return INFO level value (1)
    module.add_function("vp_logging_info_level", level_type, None);

    // vp_logging_warning_level - Return WARNING level value (2)
    module.add_function("vp_logging_warning_level", level_type, None);

    // vp_logging_error_level - Return ERROR level value (3)
    module.add_function("vp_logging_error_level", level_type, None);

    // vp_logging_critical_level - Return CRITICAL level value (4)
    module.add_function("vp_logging_critical_level", level_type, None);

    // vp_logging_notset_level - Return NOTSET level value (5)
    module.add_function("vp_logging_notset_level", level_type, None);

    Ok(())
}
