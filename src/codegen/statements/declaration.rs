use crate::ast::Expr;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarType};

/// Generate variable declaration
pub(crate) fn generate_declare<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    mutable: bool,
    value: &Option<Expr>,
) -> Result<(), String> {
    if let Some(expr) = value {
        let val = crate::codegen::expressions::generate_expr(state, expr)?;
        let ty = val.get_type();

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

        // Lists are ALWAYS heap-allocated with ARC since they can be mutated via method calls
        // This prevents stack allocation issues with in-place mutations like sort() and reverse()
        let can_stack_alloc = if is_list { false } else { state.can_stack_allocate(name) };

        // Determine if this is a reference type (pointer)
        // Chan[T] and WaitGroup are always pointer types
        let is_ref_type = val.is_pointer_value();

        // Set reference type flag in escape analyzer
        state.set_reference_type(name, is_ref_type);

        // Check if this is a Bytes literal
        let is_bytes = matches!(expr, Expr::Bytes(_, _));

        let var_type = if is_bytes {
            VarType::Bytes
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
        let is_scalar = !is_ref_type;
        let use_stack = !can_stack_alloc || is_scalar || mutable;

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
            if is_ref_type && state.needs_arc(name) {
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
