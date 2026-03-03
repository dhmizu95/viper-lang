use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarType};

/// Check if an expression is a BigInt expression
fn is_bigint_expr(expr: &Expr, state: &CodeGenState) -> bool {
    match expr {
        Expr::BigInt(..) => true,
        Expr::Ident(name, _) => state.is_bigint(name),
        Expr::BinOp { left, right, .. } => {
            is_bigint_expr(left, state) || is_bigint_expr(right, state)
        }
        Expr::Call { func, .. } => {
            if let Expr::Ident(name, _) = func.as_ref() {
                // Check for built-in BigInt functions (case insensitive for constructor)
                name == "bigint" || name == "BigInt" || name == "abs_bigint" || name == "pow_bigint" 
                    || name == "sqrt_bigint" || name == "min_bigint" || name == "max_bigint" 
                    || name == "is_zero_bigint" || name == "is_negative_bigint" 
                    || name == "sign_bigint" || name == "bit_length_bigint" || name == "pow"
            } else {
                false
            }
        }
        Expr::UnaryOp { operand, .. } => is_bigint_expr(operand, state),
        Expr::AssignmentExpr { value, .. } => is_bigint_expr(value, state),
        Expr::Conditional { then_expr, else_expr, .. } => {
            is_bigint_expr(then_expr, state) || is_bigint_expr(else_expr, state)
        }
        _ => false,
    }
}

/// Generate variable declaration
pub(crate) fn generate_declare<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    mutable: bool,
    value: &Option<Expr>,
    type_ann: &Option<Type>,
) -> Result<(), String> {
    if let Some(expr) = value {
        let mut val = crate::codegen::expressions::generate_expr(state, expr)?;

        // Convert to BigInt if type annotation is BigInt but value is not
        if matches!(type_ann, Some(Type::BigInt)) && !val.is_pointer_value() {
            // Convert i64 to BigInt using vp_bigint_from_i64
            let bigint_from_i64 = state
                .module
                .get_function("vp_bigint_from_i64")
                .ok_or_else(|| "vp_bigint_from_i64 not declared".to_string())?;
            let i64_val = val.into_int_value();
            val = state
                .ir_builder
                .build_call(state.builder, bigint_from_i64, &[i64_val.into()], "bigint_from_i64")
                .expect("bigint_from_i64 call");
        }

        // Determine the LLVM type, considering type annotations
        let ty = if matches!(type_ann, Some(Type::BigInt)) {
            // BigInt values are pointers
            state.context.ptr_type(inkwell::AddressSpace::default()).into()
        } else {
            val.get_type()
        };

        // Track BigInt variables - use comprehensive detection
        let is_bigint = matches!(type_ann, Some(Type::BigInt))
            || is_bigint_expr(expr, state);
        if is_bigint {
            state.mark_as_bigint(name.to_string());
        }

        // For BigInt values, determine if this is a "fresh" allocation (from literal/conversion)
        // or an existing reference. Fresh allocations already have ref_count=1 and don't need retain.
        let is_fresh_bigint = is_bigint && !matches!(expr, Expr::Ident(..));

        // Track list variables
        // Check for explicit list expressions, list comprehensions, or variables that hold lists
        let is_list = match expr {
            Expr::List { .. } => true,
            Expr::ListComprehension { .. } => true,
            Expr::Ident(other, _) => state.is_list(other),
            // Check for list repetition: [elem] * n
            Expr::BinOp { op: crate::ast::BinOp::Mul, left, .. } => {
                matches!(left.as_ref(), Expr::List { .. } | Expr::Array { .. })
            }
            Expr::Call { func, .. } => {
                // Check if calling a list-returning function
                // Lists return pointers, but so do strings - need to distinguish
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    // Built-in list functions
                    if func_name == "vp_list_create"
                        || func_name == "vp_list_create_f64"
                        || func_name == "vp_list_create_with_capacity"
                    {
                        true
                    // Built-in string functions - not lists
                    } else if func_name.starts_with("vp_str_") {
                        false
                    // User-defined functions - check if return value is a pointer
                    // (lists return pointers, and we can't easily determine the return type)
                    } else {
                        val.is_pointer_value()
                    }
                } else {
                    // For non-ident function calls (method calls, etc.), check if pointer
                    val.is_pointer_value()
                }
            }
            _ => false,
        };

        // Store the type annotation or inferred type in var_types for future lookups
        if let Some(ref ty) = type_ann {
            state.var_types.insert(name.to_string(), ty.clone());
        } else {
            let inferred_type = crate::codegen::expressions::core::infer_expr_type(expr);
            if inferred_type != crate::ast::Type::Infer {
                state.var_types.insert(name.to_string(), inferred_type);
            }
        }

        if is_list {
            state.mark_as_list(name.to_string());
        }

        // Track dict variables
        let is_dict = match expr {
            Expr::Dict { .. } => true,
            Expr::Ident(other, _) => state.is_dict(other),
            Expr::Call { func, .. } => {
                if let Expr::Ident(func_name, _) = func.as_ref() {
                    func_name == "vp_dict_create" || func_name == "vp_dict_create_with_capacity"
                } else {
                    false
                }
            }
            _ => false,
        };
        if is_dict {
            state.mark_as_dict(name.to_string());
        }

        // Track tuple variables - they are now heap-allocated pointers
        let is_tuple = matches!(expr, Expr::Tuple { .. });
        if is_tuple {
            state.mark_as_list(name.to_string());  // Use list tracking for ARC
        }

        // Lists can be stack-allocated if they don't escape the function
        // This is safe because:
        // 1. Non-escaping lists are only used within the function
        // 2. We insert ARC cleanup at function exit for escaping lists
        // 3. Stack allocation is faster and reduces GC pressure
        let can_stack_alloc = state.can_stack_allocate(name);

        // Determine if this is a reference type (pointer)
        // Lists, Chan[T], and WaitGroup are always pointer types
        let is_ref_type = val.is_pointer_value();

        // Set reference type flag in escape analyzer
        state.set_reference_type(name, is_ref_type);

        // Check if this is a Bytes literal
        let is_bytes = matches!(expr, Expr::Bytes(_, _));

        // Tuples are now heap-allocated pointers, treat them as Pointer type
        let is_tuple = matches!(expr, Expr::Tuple { .. });

        let var_type = if is_bytes {
            VarType::Bytes
        } else if is_tuple {
            VarType::Pointer  // Tuples are now heap-allocated pointers
        } else if val.is_float_value() {
            VarType::Float
        } else if val.is_pointer_value() {
            VarType::Pointer
        } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
            VarType::Bool
        } else {
            VarType::Int
        };

        // For scalar types (int, float), use stack allocation if mutable
        // to allow reassignment in loops
        // CRITICAL: BigInt values MUST remain as alloca (not promoted to SSA)
        // because ARC retain/release operations don't work correctly with PHI nodes
        // Tuples are now pointer types and follow pointer allocation rules
        let is_scalar = !is_ref_type && !is_tuple && var_type != VarType::Pointer;
        // Tuples use stack allocation for the pointer, but the data is heap-allocated
        let use_stack = (!can_stack_alloc || is_scalar || mutable || is_bigint) && var_type != VarType::Pointer;

        if !use_stack {
            // Use SSA register allocation for non-escaping variables or non-mutable scalars
            state.variables.insert(name.to_string(), VarInfo::new_register(val, var_type));
        } else {
            // Use stack allocation (alloca) for escaping variables or mutable scalars
            // Create alloca in function entry block to satisfy LLVM dominance
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let entry_block = func.get_first_basic_block().unwrap();
            let old_builder_pos = state.builder.get_insert_block();

            match entry_block.get_first_instruction() {
                Some(first_instr) => state.builder.position_before(&first_instr),
                None => state.builder.position_at_end(entry_block),
            }
            let alloca = state.builder.build_alloca(ty, name).expect("alloca");

            // Restore builder position
            if let Some(pos) = old_builder_pos {
                state.builder.position_at_end(pos);
            }

            state.builder.build_store(alloca, val).expect("store");
            state.variables.insert(name.to_string(), VarInfo::new_stack(alloca, var_type));

            // Insert ARC retain if this is a reference type that escapes
            // Exception: Fresh BigInt allocations already have ref_count=1
            if is_ref_type && state.needs_arc(name) && !is_fresh_bigint {
                state.build_retain(val, name);
            }
        }
    }
    Ok(())
}

/// Generate global variable declaration
/// Python syntax: global x, y, z (inside function to mark module-level variables)
pub(crate) fn generate_global<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    names: &[String],
) -> Result<(), String> {
    // The 'global' keyword marks variables as referring to module-level scope
    // We need to track these so that assignments use the global variable
    // For now, we just ensure the variables exist in global_constants
    for name in names {
        if !state.global_constants.contains_key(name) {
            // Create an uninitialized global variable (i64 default)
            let i64_type = state.context.i64_type();
            let global = state.module.add_global(i64_type, None, name);
            global.set_constant(false);
            global.set_initializer(&i64_type.const_int(0, false));
            state.global_constants.insert(name.clone(), global);
        }
    }
    Ok(())
}

/// Generate nonlocal variable declaration
/// Python syntax: nonlocal x, y (inside nested function to refer to enclosing scope)
pub(crate) fn generate_nonlocal<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    names: &[String],
) -> Result<(), String> {
    // The 'nonlocal' keyword marks variables as referring to enclosing (non-global) scope
    // This is used in nested functions to modify variables from the outer function
    // For now, we track these variables so they're looked up in the closure environment
    // Full implementation requires closure support with cell variables
    for name in names {
        // Mark variable as nonlocal - it should be looked up in the enclosing scope
        // This is a placeholder - full implementation needs closure support
        eprintln!("Warning: nonlocal '{}' - closure support is limited", name);
        
        // For now, treat nonlocal like global but search enclosing function scope
        // This will work for simple cases but not full closure semantics
        if !state.variables.contains_key(name) {
            // Create a placeholder variable that will be resolved at runtime
            // This is a simplification - proper implementation needs closure cells
            let i64_type = state.context.i64_type();
            let alloca = state.builder.build_alloca(i64_type, name)
                .map_err(|e| format!("Failed to create alloca: {:?}", e))?;
            state.variables.insert(name.clone(), VarInfo::new_stack(alloca, VarType::Int));
        }
    }
    Ok(())
}

/// Generate constant declaration
pub(crate) fn generate_const<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    value: &Expr,
) -> Result<(), String> {
    let val = crate::codegen::expressions::generate_expr(state, value)?;
    let ty = val.get_type();

    // Create a true constant (immutable global)
    let global = state.module.add_global(ty, None, name);
    global.set_constant(true); // Immutable
    global.set_initializer(&val);
    global.set_unnamed_addr(false);

    state.global_constants.insert(name.to_string(), global);
    Ok(())
}

/// Generate tuple unpacking: a, b, c = tuple_expr
pub(crate) fn generate_tuple_unpack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
    value: &Expr,
) -> Result<(), String> {
    // Generate the value expression
    let val = crate::codegen::expressions::generate_expr(state, value)?;

    // For now, we support unpacking from tuple literals and function calls returning tuples
    // Python-style: a, b = (1, 2) or a, b = get_tuple()
    if let Expr::Tuple { elements: value_elements, .. } = value {
        // Unpacking from a tuple literal
        if elements.len() != value_elements.len() {
            return Err(format!(
                "Tuple unpacking error: expected {} values, got {}",
                elements.len(),
                value_elements.len()
            ));
        }

        // Assign each element
        for (i, target) in elements.iter().enumerate() {
            let elem_val = crate::codegen::expressions::generate_expr(state, &value_elements[i])?;
            if let Expr::Ident(name, _) = target {
                // Simple variable assignment
                let ty = elem_val.get_type();
                let var_type = if elem_val.is_float_value() {
                    VarType::Float
                } else if elem_val.is_pointer_value() {
                    VarType::Pointer
                } else if elem_val.is_int_value()
                    && elem_val.get_type().into_int_type().get_bit_width() == 1
                {
                    VarType::Bool
                } else {
                    VarType::Int
                };

                // Use stack allocation for simplicity
                let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
                let entry_block = func.get_first_basic_block().unwrap();
                let old_builder_pos = state.builder.get_insert_block();

                match entry_block.get_first_instruction() {
                    Some(first_instr) => state.builder.position_before(&first_instr),
                    None => state.builder.position_at_end(entry_block),
                }
                let alloca = state.builder.build_alloca(ty, name).expect("alloca");

                if let Some(pos) = old_builder_pos {
                    state.builder.position_at_end(pos);
                }

                state.builder.build_store(alloca, elem_val).expect("store");
                state.variables.insert(name.clone(), VarInfo::new_stack(alloca, var_type));
            } else {
                return Err("Tuple unpacking only supports simple variables".to_string());
            }
        }
    } else if val.is_pointer_value() {
        // Unpacking from a function call or other expression
        // For now, we'll handle this by extracting elements via GEP if it's a struct
        // This is a simplified implementation - full support would need more work
        return Err("Tuple unpacking from non-literal tuples not yet fully supported".to_string());
    } else {
        return Err("Tuple unpacking requires a tuple value".to_string());
    }

    Ok(())
}
