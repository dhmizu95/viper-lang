//! Function declaration for Viper code generation

use crate::ast::{Param, Type};
use inkwell::context::Context;
use inkwell::values::FunctionValue;
use inkwell::types::BasicType;
use std::collections::HashMap;

use crate::codegen::types::TypeMapper;

/// Declare a function (forward declaration)
pub fn declare_function<'ctx>(
    context: &'ctx Context,
    module: &mut inkwell::module::Module<'ctx>,
    type_mapper: &TypeMapper<'ctx>,
    functions: &mut HashMap<String, FunctionValue<'ctx>>,
    name: &str,
    params: &[Param],
    return_type: &Option<Type>,
) -> Result<(), String> {
    let param_types: Vec<_> = params
        .iter()
        .map(|p| {
            let ty = p.type_ann.clone().unwrap_or(Type::I64);
            type_mapper.llvm_type(&ty).as_basic_type_enum().into()
        })
        .collect();

    let fn_type = match type_mapper.llvm_return_type(return_type) {
        Some(return_ty) => return_ty.fn_type(&param_types, false),
        None => context.void_type().fn_type(&param_types, false),
    };

    let func = module.add_function(name, fn_type, None);
    functions.insert(name.to_string(), func);

    Ok(())
}
