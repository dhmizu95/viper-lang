//! Constant and builtin handling methods

use crate::codegen::core::context::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    /// Generate __name__ builtin constant
    /// For the main module, use "__main__"; for imported modules, use the module name
    pub(crate) fn generate_name_builtin(&mut self) -> crate::codegen::Result<()> {
        // Create a global pointer variable for __name__ (will be initialized in viper_init)
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let global = self.module.add_global(ptr_type, None, "__name__");
        global.set_constant(false);
        global.set_initializer(&ptr_type.const_null());
        global.set_unnamed_addr(false);

        // Store the pointer type for correct loading later
        self.var_types.insert("__name__".to_string(), crate::ast::Type::Str);

        // Store in global_constants for lookup
        self.global_constants.insert("__name__".to_string(), global);

        Ok(())
    }

    /// Initialize __name__ builtin in viper_init
    pub(crate) fn initialize_name_builtin(&mut self) -> crate::codegen::Result<()> {
        // For the main module, use "__main__" as the name
        // This allows if __name__ == "__main__" to work correctly
        let name_value = "__main__";

        // Create string constant for __name__
        let str_val = self.ir_builder.string_const(&self.module, name_value);
        let create_func = self
            .module
            .get_function("vp_str_create")
            .ok_or_else(|| "vp_str_create not declared".to_string())?;

        let result = self
            .ir_builder
            .build_call(&mut self.builder, create_func, &[str_val.into()], "__name__")
            .ok_or_else(|| "Failed to create __name__ string".to_string())?;

        // Store the result in the __name__ global
        if let Some(global) = self.global_constants.get("__name__") {
            self.builder
                .build_store(global.as_pointer_value(), result)
                .map_err(|e| format!("Failed to store __name__: {:?}", e))?;
        }

        Ok(())
    }
}
