use crate::ast::MatchPattern;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarType};

pub(crate) fn generate_match_pattern<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    pattern: &MatchPattern,
    subject_val: inkwell::values::BasicValueEnum<'ctx>,
) -> crate::codegen::Result<inkwell::values::IntValue<'ctx>> {
    match pattern {
        MatchPattern::Wildcard => Ok(state.context.bool_type().const_int(1, false)),
        MatchPattern::Constant(expr) => {
            let const_val = crate::codegen::expressions::generate_expr(state, expr)?;

            // If subject is a pointer, load the value (handle multiple levels of indirection)
            let subject_int = if subject_val.is_pointer_value() {
                let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                // Keep loading until we get a non-pointer value
                let mut current = subject_val;
                let mut count = 0;
                while current.is_pointer_value() && count < 10 {
                    current = state
                        .builder
                        .build_load(
                            ptr_type,
                            current.into_pointer_value(),
                            &format!("load_subject_{}", count),
                        )
                        .unwrap();
                    count += 1;
                }
                current.into_int_value()
            } else {
                subject_val.into_int_value()
            };

            let const_int = if const_val.is_pointer_value() {
                let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                let loaded = state
                    .builder
                    .build_load(ptr_type, const_val.into_pointer_value(), "load_const")
                    .unwrap();
                loaded.into_int_value()
            } else {
                const_val.into_int_value()
            };

            let cmp = state
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, subject_int, const_int, "pattern_eq")
                .unwrap();
            Ok(cmp)
        }
        MatchPattern::Variable(name) => {
            let alloca = state.builder.build_alloca(subject_val.get_type(), name).unwrap();
            state.builder.build_store(alloca, subject_val).unwrap();
            state.variables.insert(name.clone(), VarInfo::new_stack(alloca, VarType::Pointer));
            Ok(state.context.bool_type().const_int(1, false))
        }
        MatchPattern::Tuple(_) => Ok(state.context.bool_type().const_int(1, false)),
        MatchPattern::List { .. } => Ok(state.context.bool_type().const_int(1, false)),
        MatchPattern::TypeCheck { type_name, binding } => {
            // For Ok/Err patterns on Result types, check the is_ok field
            if type_name == "Ok" || type_name == "Err" {
                // subject_val should be a Result struct { is_ok: i8, value: i64 }
                // Note: values are stored as i64 (or pointer bitcast to i64)
                let result_struct = if subject_val.is_pointer_value() {
                    // Load the struct from alloca
                    let result_struct_type = state.context.struct_type(
                        &[state.context.i8_type().into(), state.context.i64_type().into()],
                        false,
                    );
                    state
                        .builder
                        .build_load(
                            result_struct_type,
                            subject_val.into_pointer_value(),
                            "result_loaded",
                        )
                        .map_err(|e| format!("Failed to load Result: {:?}", e))?
                } else {
                    subject_val
                }
                .into_struct_value();

                // Extract is_ok field
                let is_ok_val = state
                    .builder
                    .build_extract_value(result_struct, 0, "is_ok")
                    .map_err(|e| format!("Failed to extract is_ok: {:?}", e))?
                    .into_int_value();

                // Check if is_ok matches the pattern
                let expected = if type_name == "Ok" { 1 } else { 0 };
                let matches = state
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::EQ,
                        is_ok_val,
                        state.context.i8_type().const_int(expected as u64, false),
                        "check_ok",
                    )
                    .unwrap();

                // If there's a binding, extract and bind the value
                if let Some(binding_name) = binding {
                    // Extract value field (second field) - stored as i64 (tagged int)
                    let value_i64 = state
                        .builder
                        .build_extract_value(result_struct, 1, "value_i64")
                        .map_err(|e| format!("Failed to extract value: {:?}", e))?
                        .into_int_value();

                    // Allocate and store the i64 value
                    let i64_type = state.context.i64_type();
                    let alloca = state.builder.build_alloca(i64_type, binding_name).unwrap();
                    state.builder.build_store(alloca, value_i64).unwrap();
                    // Tagged int values use VarType::Int for proper handling
                    state
                        .variables
                        .insert(binding_name.clone(), VarInfo::new_stack(alloca, VarType::Int));
                }

                Ok(matches)
            } else {
                // For other type checks, just return true for now
                // A full implementation would check runtime type information
                Ok(state.context.bool_type().const_int(1, false))
            }
        }
        MatchPattern::Range { .. } => Ok(state.context.bool_type().const_int(1, false)),
    }
}
