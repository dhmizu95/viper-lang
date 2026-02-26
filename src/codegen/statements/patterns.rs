use crate::ast::MatchPattern;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarType};

pub(crate) fn generate_match_pattern<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    pattern: &MatchPattern,
    subject_val: inkwell::values::BasicValueEnum<'ctx>,
) -> Result<inkwell::values::IntValue<'ctx>, String> {
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
        MatchPattern::TypeCheck { .. } => Ok(state.context.bool_type().const_int(1, false)),
        MatchPattern::Range { .. } => Ok(state.context.bool_type().const_int(1, false)),
    }
}
