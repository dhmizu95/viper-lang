//! super() method call code generation

use crate::ast::Expr;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;

/// Generate super().method() call - resolves method through MRO
pub fn generate_super_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    method_name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Get the current class from state (set when generating class methods)
    let class_name = state.current_class.clone()
        .ok_or_else(|| "super() can only be used inside a class method".to_string())?;

    // Get the current class metadata
    let metadata = crate::codegen::oop::with_class_registry(|reg| {
        reg.get_class(&class_name).cloned()
    }).ok_or_else(|| format!("Class '{}' not found", class_name))?;

    // Find the method in parent classes via MRO
    // Skip the first entry in MRO (which is the current class itself)
    let mut found_method = None;
    for mro_class_name in metadata.mro.iter().skip(1) {
        if let Some(method) = crate::codegen::oop::with_class_registry(|reg| {
            reg.get_class(mro_class_name).and_then(|c| c.get_method(method_name).cloned())
        }) {
            found_method = Some(method);
            break;
        }
    }

    let method = found_method
        .ok_or_else(|| format!("Method '{}' not found in parent classes", method_name))?;

    // Get self from the function's first parameter
    let current_function = state.builder.get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| "Not inside a function".to_string())?;

    let self_ptr = current_function.get_nth_param(0)
        .ok_or_else(|| "Method should have self parameter".to_string())?
        .into_pointer_value();

    // Build argument list: self + user args
    let mut arg_values: Vec<_> = args.iter()
        .map(|a| crate::codegen::expressions::generate_expr(state, a)
            .map(|v| inkwell::values::BasicMetadataValueEnum::from(v)))
        .collect::<Result<_, _>>()?;

    // Insert self as first argument
    arg_values.insert(0, self_ptr.into());

    // Call the parent method
    if let Some(func_val) = state.functions.get(&method.mangled_name).copied() {
        let result = state.ir_builder.build_call(
            state.builder,
            func_val,
            &arg_values,
            &format!("super_call_{}", method_name),
        );

        Ok(result.unwrap_or(state.context.i64_type().const_int(0, false).into()))
    } else {
        Err(format!("Parent method '{}' not found", method_name))
    }
}
