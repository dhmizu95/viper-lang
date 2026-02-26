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

        // Use escape analysis to determine allocation strategy
        let can_stack_alloc = state.can_stack_allocate(name);

        // Determine if this is a reference type (pointer)
        // Chan[T] and WaitGroup are always pointer types
        let is_ref_type = val.is_pointer_value();

        // Set reference type flag in escape analyzer
        state.set_reference_type(name, is_ref_type);

        let var_type = if val.is_float_value() {
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
