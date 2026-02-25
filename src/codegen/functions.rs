//! Function declaration for Viper code generation

use crate::ast::{Param, Type};
use crate::utils::mangle_function_name;
use inkwell::context::Context;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
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
    let param_types: Vec<Type> = params
        .iter()
        .map(|p| p.type_ann.clone().unwrap_or(Type::I64))
        .collect();

    let param_llvm_types: Vec<_> = param_types
        .iter()
        .map(|ty| type_mapper.llvm_type(ty).as_basic_type_enum().into())
        .collect();

    let fn_type = match type_mapper.llvm_return_type(return_type) {
        Some(return_ty) => return_ty.fn_type(&param_llvm_types, false),
        None => context.void_type().fn_type(&param_llvm_types, false),
    };

    let mangled_name = mangle_function_name(name, &param_types);
    let func = module.add_function(&mangled_name, fn_type, None);
    functions.insert(mangled_name, func);

    Ok(())
}
