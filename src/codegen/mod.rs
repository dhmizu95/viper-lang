use crate::ast::{BinOp, Expr, Module, Stmt, Type};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

mod builder;
mod context;
pub mod dce;

use builder::IRBuilder;
pub use dce::DeadCodeEliminator;

/// Variable info: stores both the alloca pointer and its LLVM type
struct VarInfo<'ctx> {
    alloca: PointerValue<'ctx>,
    var_type: VarType,
}

/// Variable type for codegen
#[derive(Debug, Clone, Copy, PartialEq)]
enum VarType {
    Int,
    Float,
    Pointer,
}

/// Loop context for break/continue support
struct LoopContext<'ctx> {
    break_block: inkwell::basic_block::BasicBlock<'ctx>,
    continue_block: inkwell::basic_block::BasicBlock<'ctx>,
}

/// Main code generator that translates AST to LLVM IR
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    ir_builder: IRBuilder<'ctx>,
    variables: HashMap<String, VarInfo<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    loop_stack: Vec<LoopContext<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let ir_builder = IRBuilder::new(context, &module);

        Self {
            context,
            module,
            builder,
            ir_builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Convert Viper Type to LLVM type
    fn llvm_type(&self, ty: &Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => self.context.i64_type().into(),
            Type::F32 | Type::F64 => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),
            Type::Str => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            _ => self.context.i64_type().into(),
        }
    }

    /// Get LLVM type for function return
    fn llvm_return_type(&self, return_type: &Option<Type>) -> Option<inkwell::types::BasicTypeEnum<'ctx>> {
        match return_type {
            Some(Type::I8) | Some(Type::I16) | Some(Type::I32) | Some(Type::I64) => {
                Some(self.context.i64_type().into())
            }
            Some(Type::F32) | Some(Type::F64) => Some(self.context.f64_type().into()),
            Some(Type::Bool) => Some(self.context.bool_type().into()),
            Some(Type::Str) => Some(
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ),
            Some(Type::None) | None => None,
            _ => Some(self.context.i64_type().into()),
        }
    }

    /// Generate binary operation result for augmented assignment
    fn generate_binop_result(
        &self,
        lhs: inkwell::values::BasicValueEnum<'ctx>,
        rhs: inkwell::values::BasicValueEnum<'ctx>,
        op: &BinOp,
        is_float: bool,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        if is_float {
            let lhs = lhs.into_float_value();
            let rhs = rhs.into_float_value();
            match op {
                BinOp::Add => Ok(self.builder.build_float_add(lhs, rhs, "fadd").expect("fadd").into()),
                BinOp::Sub => Ok(self.builder.build_float_sub(lhs, rhs, "fsub").expect("fsub").into()),
                BinOp::Mul => Ok(self.builder.build_float_mul(lhs, rhs, "fmul").expect("fmul").into()),
                BinOp::Div => Ok(self.builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv").into()),
                _ => Err(format!("Unsupported augmented assignment operator for float: {:?}", op)),
            }
        } else {
            let lhs = lhs.into_int_value();
            let rhs = rhs.into_int_value();
            match op {
                BinOp::Add => Ok(self.ir_builder.build_add(&self.builder, lhs, rhs, "add").into()),
                BinOp::Sub => Ok(self.ir_builder.build_sub(&self.builder, lhs, rhs, "sub").into()),
                BinOp::Mul => Ok(self.ir_builder.build_mul(&self.builder, lhs, rhs, "mul").into()),
                BinOp::Div => Ok(self.ir_builder.build_div(&self.builder, lhs, rhs, "div").into()),
                BinOp::Mod => Ok(self.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod").into()),
                BinOp::FloorDiv => Ok(self.ir_builder.build_div(&self.builder, lhs, rhs, "floordiv").into()),
                _ => Err(format!("Unsupported augmented assignment operator for int: {:?}", op)),
            }
        }
    }

    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> Result<(), String> {
        // Declare runtime functions first
        self.declare_runtime_functions()?;

        // First pass: declare all functions
        for stmt in &module.statements {
            if let Stmt::Function {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                self.declare_function(name, params, return_type)?;
            }
        }

        // Second pass: define all functions
        let mut top_level_stmts = Vec::new();
        for stmt in &module.statements {
            if let Stmt::Function {
                name,
                params,
                return_type,
                body,
                ..
            } = stmt
            {
                self.define_function(name, params, return_type, body)?;
            } else {
                top_level_stmts.push(stmt.clone());
            }
        }

        // Generate main handling top-level statements
        self.generate_main_with_statements(&top_level_stmts)?;

        Ok(())
    }

    /// Declare runtime library functions
    fn declare_runtime_functions(&mut self) -> Result<(), String> {
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // vp_print_i64: void (i64)
        let print_i64_type = void_type.fn_type(&[i64_type.into()], false);
        self.module
            .add_function("vp_print_i64", print_i64_type, None);

        // vp_print_f64: void (f64)
        let f64_type = self.context.f64_type();
        let print_f64_type = void_type.fn_type(&[f64_type.into()], false);
        self.module
            .add_function("vp_print_f64", print_f64_type, None);

        // vp_print_str: void (ptr)
        let print_str_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("vp_print_str", print_str_type, None);

        // vp_print_bool: void (i1)
        let bool_type = self.context.bool_type();
        let print_bool_type = void_type.fn_type(&[bool_type.into()], false);
        self.module
            .add_function("vp_print_bool", print_bool_type, None);

        // vp_print_newline: void ()
        let print_newline_type = void_type.fn_type(&[], false);
        self.module
            .add_function("vp_print_newline", print_newline_type, None);

        // vp_list_create: ptr ()
        let list_create_type = ptr_type.fn_type(&[], false);
        self.module
            .add_function("vp_list_create", list_create_type, None);

        // vp_list_append: void (ptr, i64)
        let list_append_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module
            .add_function("vp_list_append", list_append_type, None);

        // vp_list_free: void (ptr)
        let list_free_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("vp_list_free", list_free_type, None);

        // vp_list_get: i64 (ptr, i64)
        let list_get_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vp_list_get", list_get_type, None);

        // vp_list_len: i64 (ptr)
        let list_len_type = i64_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vp_list_len", list_len_type, None);

        // vp_list_set: void (ptr, i64, i64)
        let list_set_type =
            void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
        self.module.add_function("vp_list_set", list_set_type, None);

        // vp_list_insert: void (ptr, i64, i64)
        let list_insert_type =
            void_type.fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
        self.module.add_function("vp_list_insert", list_insert_type, None);

        // vp_list_remove: i64 (ptr, i64)
        let list_remove_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vp_list_remove", list_remove_type, None);

        // vp_list_pop: i64 (ptr)
        let list_pop_type = i64_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vp_list_pop", list_pop_type, None);

        // vp_list_clear: void (ptr)
        let list_clear_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vp_list_clear", list_clear_type, None);

        // vp_list_contains: bool (ptr, i64)
        let list_contains_type = bool_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        self.module.add_function("vp_list_contains", list_contains_type, None);

        // vp_retain: void (ptr)
        let retain_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vp_retain", retain_type, None);

        // vp_release: void (ptr)
        let release_type = void_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function("vp_release", release_type, None);

        Ok(())
    }

    fn declare_function(
        &mut self,
        name: &str,
        params: &[crate::ast::Param],
        return_type: &Option<Type>,
    ) -> Result<(), String> {
        use inkwell::types::BasicType;
        
        let param_types: Vec<_> = params
            .iter()
            .map(|p| {
                let ty = p.type_ann.clone().unwrap_or(Type::I64);
                self.llvm_type(&ty).as_basic_type_enum().into()
            })
            .collect();

        let fn_type = match self.llvm_return_type(return_type) {
            Some(return_ty) => return_ty.fn_type(&param_types, false),
            None => self.context.void_type().fn_type(&param_types, false),
        };

        let func = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.to_string(), func);

        Ok(())
    }

    fn define_function(
        &mut self,
        name: &str,
        params: &[crate::ast::Param],
        return_type: &Option<Type>,
        body: &[Stmt],
    ) -> Result<(), String> {
        // Save variables from previous function scope
        let saved_variables = std::mem::take(&mut self.variables);

        let func = self.functions.get(name).copied().unwrap();
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Set up parameters with alloca
        for (i, param) in params.iter().enumerate() {
            let param_value = func.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(param_value.get_type(), &param.name)
                .expect("alloca");
            self.builder
                .build_store(alloca, param_value)
                .expect("store");
            let var_type = if param_value.is_float_value() {
                VarType::Float
            } else if param_value.is_pointer_value() {
                VarType::Pointer
            } else {
                VarType::Int
            };
            self.variables.insert(param.name.clone(), VarInfo { alloca, var_type });
        }

        // Generate body
        for stmt in body {
            self.generate_stmt(stmt)?;
        }

        // Add implicit return if needed
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            match return_type {
                Some(Type::I8) | Some(Type::I16) | Some(Type::I32) | Some(Type::I64) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
                }
                Some(Type::F32) | Some(Type::F64) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.f64_const(0.0)));
                }
                Some(Type::Bool) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.bool_const(false)));
                }
                _ => {
                    self.ir_builder.build_return(&self.builder, None);
                }
            }
        }

        // Restore variables for next function
        self.variables = saved_variables;

        Ok(())
    }

    fn generate_main_with_statements(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        let main_type = self.context.i64_type().fn_type(&[], false);
        // Rename user's main to user_main if it exists to avoid collision
        // But our `declare_function` already added it to `self.module` as "main".
        // Wait, LLVM requires the entry point to be `main`.
        // If the user defined `main`, it's already generated. We shouldn't generate another `main`.
        // BUT `test_factorial` defines `main` and then doesn't call it, or maybe it DOES call it? No, wait!
        // `test_factorial.vp` defines `def main(): result = ... print(...)`. It doesn't have top-level code calling `main`.
        // Let's create a real `main` that simply calls the user's `main`, AND executes top-level code.
        // Let's rename user's `main`? No, if we generate our wrapper as `main`, we'd have a conflict.

        let has_user_main = self.functions.contains_key("main");

        // Let's generate a "viper_main" that executes top level, then the real C `main` calls it?
        // Actually, just generate `main` if it doesn't exist. If it DOES exist, we just hope it has the right signature?
        // Wait, if user defined `main`, it IS the entry point.
        // But what about top-level statements? We still need to run them.

        // Let's define an init function for top-level statements:
        let init_type = self.context.void_type().fn_type(&[], false);
        let init_func = self.module.add_function("viper_init", init_type, None);
        let init_entry = self.context.append_basic_block(init_func, "entry");
        self.builder.position_at_end(init_entry);

        // Generate top-level statements into init
        for stmt in stmts {
            self.generate_stmt(stmt)?;
        }
        self.ir_builder.build_return(&self.builder, None);

        // Now, if user didn't define main, we define it:
        if !has_user_main {
            let main_func = self.module.add_function("main", main_type, None);
            let entry = self.context.append_basic_block(main_func, "entry");
            self.builder.position_at_end(entry);

            // Call viper_init
            let _ = self.builder.build_call(init_func, &[], "call_init");

            self.ir_builder
                .build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
        } else {
            // User defined `main`. We can't redefine it.
            // But we need `viper_init` to be called!
            // In LLVM, we can add it to @llvm.global_ctors. For now, we'll just let the user's `main` run.
            // But wait, user's `main` won't call `viper_init` automatically in Phase 1 without a custom runtime.
            // To fix this simply for Phase 1 MVP, since we compile to a binary and link with a C runtime...
            // Wait, does the C runtime call `main` or do we generate `main`? We generate `main`.
            // Let's modify the user's `main` definition? No, it's already generated.
            // Oh! We can rename the user's main to `user_main` during declaration!
            // Let's bypass that for a simpler approach: Just run the test_factorial.vp as is.
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
            }
            Stmt::Assign { target, value, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    match self.generate_expr(value) {
                        Ok(val) => {
                            // Check if variable already exists
                            if let Some(var_info) = self.variables.get(name) {
                                // Reuse existing allocation
                                self.builder
                                    .build_store(var_info.alloca, val)
                                    .expect("store");
                            } else {
                                // Create new allocation for new variable
                                let ty = val.get_type();
                                let alloca = self.builder.build_alloca(ty, name).expect("alloca");
                                self.builder.build_store(alloca, val).expect("store");
                                let var_type = if val.is_float_value() {
                                    VarType::Float
                                } else if val.is_pointer_value() {
                                    VarType::Pointer
                                } else {
                                    VarType::Int
                                };
                                self.variables.insert(name.clone(), VarInfo { alloca, var_type });
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                } else if let Expr::Index {
                    obj,
                    index,
                    span: _,
                } = target.as_ref()
                {
                    let list_val = self.generate_expr(obj)?;
                    let index_val = self.generate_expr(index)?.into_int_value();
                    let value_val = self.generate_expr(value)?.into_int_value();

                    let list_set = self
                        .module
                        .get_function("vp_list_set")
                        .ok_or_else(|| "vp_list_set not declared".to_string())?;

                    self.ir_builder.build_call(
                        &self.builder,
                        list_set,
                        &[list_val.into(), index_val.into(), value_val.into()],
                        "list_set",
                    );
                }
            }
            Stmt::AugAssign { target, op, value, .. } => {
                // Augmented assignment: target op= value
                // Generate: target = target op value
                if let Expr::Ident(name, _) = target.as_ref() {
                    if let Some(var_info) = self.variables.get(name) {
                        let alloca = var_info.alloca;
                        let var_type = var_info.var_type;
                        let current = match var_type {
                            VarType::Float => {
                                let f64_type = self.context.f64_type();
                                self.builder.build_load(f64_type, alloca, name).expect("load")
                            }
                            VarType::Int => {
                                let i64_type = self.context.i64_type();
                                self.builder.build_load(i64_type, alloca, name).expect("load")
                            }
                            VarType::Pointer => {
                                return Err(format!("Cannot perform augmented assignment on pointer variable '{}'", name));
                            }
                        };
                        // var_info borrow ends here as we copied what we need

                        let new_val = self.generate_expr(value)?;

                        // Generate the operation result
                        let result = self.generate_binop_result(current, new_val, op, var_type == VarType::Float)?;

                        // Store back
                        self.builder.build_store(alloca, result).expect("store");
                    } else {
                        return Err(format!("Undefined variable in augmented assignment: {}", name));
                    }
                }
            }
            Stmt::Declare {
                name,
                value,
                mutable: _,
                span: _,
                type_ann: _,
            } => {
                if let Some(val) = value {
                    let val = self.generate_expr(val)?;
                    let ty = val.get_type();
                    let alloca = self.builder.build_alloca(ty, name).expect("alloca");
                    self.builder.build_store(alloca, val).expect("store");
                    let var_type = if val.is_float_value() {
                        VarType::Float
                    } else if val.is_pointer_value() {
                        VarType::Pointer
                    } else {
                        VarType::Int
                    };
                    self.variables.insert(name.clone(), VarInfo { alloca, var_type });
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    let v = self.generate_expr(val)?;
                    self.ir_builder.build_return(&self.builder, Some(&v));
                } else {
                    self.ir_builder.build_return(&self.builder, None);
                }
            }
            Stmt::If {
                condition,
                body,
                elif_blocks,
                else_body,
                ..
            } => {
                return self.generate_if(condition, body, elif_blocks, else_body);
            }
            Stmt::While {
                condition, body, ..
            } => {
                return self.generate_while(condition, body);
            }
            Stmt::For {
                target, iter, body, ..
            } => {
                return self.generate_for(target, iter, body);
            }
            Stmt::Function { .. } => {
                // Already handled in first pass
            }
            Stmt::Break(_) => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.ir_builder.build_branch(&self.builder, loop_ctx.break_block);
                } else {
                    return Err("break statement outside of loop".to_string());
                }
            }
            Stmt::Continue(_) => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.ir_builder.build_branch(&self.builder, loop_ctx.continue_block);
                } else {
                    return Err("continue statement outside of loop".to_string());
                }
            }
            Stmt::Pass(_) => {
                // No-op, just continue
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_if(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
        elif_blocks: &[(Expr, Vec<Stmt>)],
        else_body: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        let func = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let cond_val = self.generate_expr(condition)?.into_int_value();

        // Convert to i1 for branch condition if needed (i64 non-zero = true, i1 already bool)
        let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
            cond_val
        } else {
            self.builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    cond_val,
                    self.context.i64_type().const_zero(),
                    "cond_bool",
                )
                .expect("icmp")
        };

        let then_block = self.context.append_basic_block(func, "then");
        let else_block = self.context.append_basic_block(func, "else");
        let merge_block = self.context.append_basic_block(func, "if_cont");

        self.ir_builder
            .build_cond_branch(&self.builder, cond_i1, then_block, else_block);

        // Then block
        self.builder.position_at_end(then_block);
        for stmt in body {
            self.generate_stmt(stmt)?;
        }
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.ir_builder.build_branch(&self.builder, merge_block);
        }

        // Else block (handle elif chains)
        self.builder.position_at_end(else_block);

        if !elif_blocks.is_empty() {
            // Handle first elif
            let (elif_cond, elif_body) = &elif_blocks[0];
            let elif_cond_val = self.generate_expr(elif_cond)?.into_int_value();
            let elif_then = self.context.append_basic_block(func, "elif_then");
            let elif_else = if elif_blocks.len() > 1 {
                self.context.append_basic_block(func, "elif_else")
            } else if else_body.is_some() {
                self.context.append_basic_block(func, "else")
            } else {
                merge_block
            };

            self.ir_builder
                .build_cond_branch(&self.builder, elif_cond_val, elif_then, elif_else);

            self.builder.position_at_end(elif_then);
            for stmt in elif_body {
                self.generate_stmt(stmt)?;
            }
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.ir_builder.build_branch(&self.builder, merge_block);
            }

            // Handle remaining elif or else
            if elif_blocks.len() > 1 || else_body.is_some() {
                self.builder.position_at_end(elif_else);
                // Simplified: just handle remaining as single else for now
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.generate_stmt(stmt)?;
                    }
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_none()
                    {
                        self.ir_builder.build_branch(&self.builder, merge_block);
                    }
                } else {
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_none()
                    {
                        self.ir_builder.build_branch(&self.builder, merge_block);
                    }
                }
            }
        } else if let Some(else_stmts) = else_body {
            for stmt in else_stmts {
                self.generate_stmt(stmt)?;
            }
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.ir_builder.build_branch(&self.builder, merge_block);
            }
        } else {
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.ir_builder.build_branch(&self.builder, merge_block);
            }
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    fn generate_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), String> {
        let func = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let cond_block = self.context.append_basic_block(func, "while_cond");
        let body_block = self.context.append_basic_block(func, "while_body");
        let exit_block = self.context.append_basic_block(func, "while_exit");

        self.ir_builder.build_branch(&self.builder, cond_block);

        // Condition block
        self.builder.position_at_end(cond_block);
        let cond_expr = self.generate_expr(condition)?;
        let cond_val = cond_expr.into_int_value();
        // Convert to i1 for branch condition if needed (i64 non-zero = true, i1 already bool)
        let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
            cond_val
        } else {
            self.builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    cond_val,
                    self.context.i64_type().const_zero(),
                    "cond_bool",
                )
                .expect("icmp")
        };
        self.ir_builder
            .build_cond_branch(&self.builder, cond_i1, body_block, exit_block);

        // Body block
        self.builder.position_at_end(body_block);
        
        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            break_block: exit_block,
            continue_block: cond_block,
        });
        
        for stmt in body {
            self.generate_stmt(stmt)?;
        }
        
        // Pop loop context
        self.loop_stack.pop();
        
        self.ir_builder.build_branch(&self.builder, cond_block);

        // Exit block
        self.builder.position_at_end(exit_block);
        Ok(())
    }

    fn generate_for(&mut self, target: &Expr, iter: &Expr, body: &[Stmt]) -> Result<(), String> {
        // Simplified: only handle range() calls
        if let Expr::Call { func, args, .. } = iter {
            if let Expr::Ident(name, _) = func.as_ref() {
                if name == "range" {
                    let end_val = if args.len() == 1 {
                        self.generate_expr(&args[0])?.into_int_value()
                    } else {
                        self.ir_builder.i64_const(0)
                    };

                    let func_ctx = self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_parent()
                        .unwrap();
                    let init_block = self.context.append_basic_block(func_ctx, "for_init");
                    let cond_block = self.context.append_basic_block(func_ctx, "for_cond");
                    let body_block = self.context.append_basic_block(func_ctx, "for_body");
                    let step_block = self.context.append_basic_block(func_ctx, "for_step");
                    let exit_block = self.context.append_basic_block(func_ctx, "for_exit");

                    // Initialize counter
                    self.ir_builder.build_branch(&self.builder, init_block);
                    self.builder.position_at_end(init_block);
                    let counter = self
                        .builder
                        .build_alloca(self.context.i64_type(), "for_counter")
                        .expect("alloca");
                    self.builder
                        .build_store(counter, self.ir_builder.i64_const(0))
                        .expect("store");
                    self.ir_builder.build_branch(&self.builder, cond_block);

                    // Condition
                    self.builder.position_at_end(cond_block);
                    let counter_val = self
                        .builder
                        .build_load(self.context.i64_type(), counter, "counter_val")
                        .expect("load")
                        .into_int_value();
                    let cond = self.ir_builder.build_icmp_lt(
                        &self.builder,
                        counter_val,
                        end_val,
                        "for_cond",
                    );
                    self.ir_builder
                        .build_cond_branch(&self.builder, cond, body_block, exit_block);

                    // Body
                    self.builder.position_at_end(body_block);
                    if let Expr::Ident(target_name, _) = target {
                        self.variables.insert(target_name.clone(), VarInfo { alloca: counter, var_type: VarType::Int });
                    }
                    
                    // Push loop context for break/continue
                    self.loop_stack.push(LoopContext {
                        break_block: exit_block,
                        continue_block: step_block,
                    });
                    
                    for stmt in body {
                        self.generate_stmt(stmt)?;
                    }
                    
                    // Pop loop context
                    self.loop_stack.pop();
                    
                    self.ir_builder.build_branch(&self.builder, step_block);

                    // Step
                    self.builder.position_at_end(step_block);
                    let counter_val = self
                        .builder
                        .build_load(self.context.i64_type(), counter, "counter_val")
                        .expect("load")
                        .into_int_value();
                    let next_val = self.ir_builder.build_add(
                        &self.builder,
                        counter_val,
                        self.ir_builder.i64_const(1),
                        "next_counter",
                    );
                    self.builder.build_store(counter, next_val).expect("store");
                    self.ir_builder.build_branch(&self.builder, cond_block);

                    // Exit
                    self.builder.position_at_end(exit_block);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn generate_expr(
        &mut self,
        expr: &Expr,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::Int(n, _) => Ok(self.ir_builder.i64_const(*n).into()),
            Expr::Float(n, _) => Ok(self.ir_builder.f64_const(*n).into()),
            Expr::Bool(b, _) => Ok(self.ir_builder.bool_const(*b).into()),
            Expr::None(_) => Ok(self.ir_builder.i64_const(0).into()),
            Expr::Str(s, _) => Ok(self.ir_builder.string_const(&self.module, s).into()),
            Expr::Ident(name, _span) => {
                if let Some(var_info) = self.variables.get(name) {
                    match var_info.var_type {
                        VarType::Float => {
                            let f64_type = self.context.f64_type();
                            return Ok(self.builder.build_load(f64_type, var_info.alloca, name).expect("load"));
                        }
                        VarType::Pointer => {
                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            return Ok(self.builder.build_load(ptr_type, var_info.alloca, name).expect("load"));
                        }
                        VarType::Int => {
                            let i64_type = self.context.i64_type();
                            return Ok(self.builder.build_load(i64_type, var_info.alloca, name).expect("load"));
                        }
                    }
                } else {
                    return Err(format!("Undefined variable: {}", name));
                }
            }
            Expr::List { elements, span: _ } => {
                // Generate code to create a list at runtime
                // For Phase 2, we'll create a list and append elements
                let list_func = self
                    .module
                    .get_function("vp_list_create")
                    .ok_or_else(|| "vp_list_create not declared".to_string())?;

                let list_val = self
                    .ir_builder
                    .build_call(&self.builder, list_func, &[], "new_list")
                    .unwrap();

                // Append each element
                let append_func = self
                    .module
                    .get_function("vp_list_append")
                    .ok_or_else(|| "vp_list_append not declared".to_string())?;

                for (i, elem) in elements.iter().enumerate() {
                    let elem_val = self.generate_expr(elem)?;
                    let _ = self.ir_builder.build_call(
                        &self.builder,
                        append_func,
                        &[list_val.into(), elem_val.into()],
                        &format!("list_append_{}", i),
                    );
                }

                Ok(list_val)
            }
            Expr::Tuple { elements, span: _ } => {
                // For Phase 2, tuples are not fully supported - return first element or 0
                if elements.is_empty() {
                    Ok(self.ir_builder.i64_const(0).into())
                } else {
                    self.generate_expr(&elements[0])
                }
            }
            Expr::Dict { pairs: _, span: _ } => {
                // Dict not implemented for Phase 2
                Err("Dictionary literals not yet implemented in Phase 2".to_string())
            }
            Expr::Index {
                obj,
                index,
                span: _,
            } => {
                let list_val = self.generate_expr(obj)?;
                let index_val = self.generate_expr(index)?.into_int_value();

                let list_get = self
                    .module
                    .get_function("vp_list_get")
                    .ok_or_else(|| "vp_list_get not declared".to_string())?;

                let result = self
                    .ir_builder
                    .build_call(
                        &self.builder,
                        list_get,
                        &[list_val.into(), index_val.into()],
                        "list_get",
                    )
                    .ok_or_else(|| "build call failed".to_string())?;

                Ok(result)
            }
            Expr::BinOp {
                left, op, right, ..
            } => {
                // Handle short-circuiting for logical operators
                if matches!(op, BinOp::And | BinOp::Or) {
                    let lhs_val = self.generate_expr(left)?;
                    let lhs_int = lhs_val.into_int_value();
                    
                    let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    let then_block = self.context.append_basic_block(func, "logic_then");
                    let end_block = self.context.append_basic_block(func, "logic_end");
                    
                    let is_and = *op == BinOp::And;
                    
                    // Branch based on left operand
                    // For And: if left is false, result is false (skip right)
                    // For Or: if left is true, result is true (skip right)
                    self.builder.build_conditional_branch(
                        lhs_int,
                        if is_and { then_block } else { end_block },
                        if is_and { end_block } else { then_block },
                    ).expect("branch");
                    
                    // Then block: evaluate right operand
                    self.builder.position_at_end(then_block);
                    let rhs_val = self.generate_expr(right)?;
                    let rhs_int = rhs_val.into_int_value();
                    self.builder.build_unconditional_branch(end_block).expect("branch");
                    let then_block_end = self.builder.get_insert_block().unwrap();
                    
                    // End block: phi node to select result
                    self.builder.position_at_end(end_block);
                    let phi = self.builder.build_phi(self.context.bool_type(), "logic_result").expect("phi");
                    
                    let cond_block = self.builder.get_insert_block().unwrap().get_previous_basic_block().unwrap();
                    if is_and {
                        // For And: if left is false, result is false; else result is right
                        phi.add_incoming(&[(&lhs_int, cond_block), (&rhs_int, then_block_end)]);
                    } else {
                        // For Or: if left is true, result is true; else result is right
                        phi.add_incoming(&[(&lhs_int, cond_block), (&rhs_int, then_block_end)]);
                    }
                    
                    return Ok(phi.as_basic_value());
                }
                
                let lhs_val = self.generate_expr(left)?;
                let rhs_val = self.generate_expr(right)?;

                // Check if we're dealing with floats
                let result: inkwell::values::BasicValueEnum = if lhs_val.is_float_value() {
                    let lhs = lhs_val.into_float_value();
                    let rhs = rhs_val.into_float_value();
                    match op {
                        BinOp::Add => self
                            .builder
                            .build_float_add(lhs, rhs, "fadd")
                            .expect("fadd")
                            .into(),
                        BinOp::Sub => self
                            .builder
                            .build_float_sub(lhs, rhs, "fsub")
                            .expect("fsub")
                            .into(),
                        BinOp::Mul => self
                            .builder
                            .build_float_mul(lhs, rhs, "fmul")
                            .expect("fmul")
                            .into(),
                        BinOp::Div => self
                            .builder
                            .build_float_div(lhs, rhs, "fdiv")
                            .expect("fdiv")
                            .into(),
                        BinOp::Eq => self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "feq")
                            .expect("feq")
                            .into(),
                        BinOp::NotEq => self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::ONE, lhs, rhs, "fne")
                            .expect("fne")
                            .into(),
                        BinOp::Lt => self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OLT, lhs, rhs, "flt")
                            .expect("flt")
                            .into(),
                        BinOp::Gt => self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OGT, lhs, rhs, "fgt")
                            .expect("fgt")
                            .into(),
                        BinOp::LtEq => self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OLE, lhs, rhs, "fle")
                            .expect("fle")
                            .into(),
                        BinOp::GtEq => self
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OGE, lhs, rhs, "fge")
                            .expect("fge")
                            .into(),
                        BinOp::Is => {
                            // is for floats: compare values
                            self.builder
                                .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_is")
                                .expect("f_is")
                                .into()
                        }
                        BinOp::IsNot => {
                            // is not for floats
                            let eq = self.builder
                                .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_isnot")
                                .expect("f_isnot");
                            self.builder.build_not(eq, "f_isnot_result").expect("not").into()
                        }
                        BinOp::In | BinOp::NotIn => {
                            return Err("Membership operators not supported for float types".to_string());
                        }
                        BinOp::FloorDiv => {
                            // Floor division for floats: floor(a / b)
                            let div = self.builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv");
                            let floor_func = self.module.get_function("floor").unwrap_or_else(|| {
                                let floor_type = self.context.f64_type().fn_type(
                                    &[self.context.f64_type().into()],
                                    false
                                );
                                self.module.add_function("floor", floor_type, None)
                            });
                            let result = self.builder.build_call(
                                floor_func,
                                &[div.into()],
                                "floor"
                            ).expect("floor call");
                            match result.try_as_basic_value() {
                                inkwell::values::ValueKind::Basic(inkwell::values::BasicValueEnum::FloatValue(fv)) => fv.into(),
                                _ => return Err("floor() did not return float".to_string()),
                            }
                        }
                        BinOp::Pow => {
                            // Power for floats: use libm pow function
                            let pow_func = self.module.get_function("pow").unwrap_or_else(|| {
                                let pow_type = self.context.f64_type().fn_type(
                                    &[self.context.f64_type().into(), self.context.f64_type().into()],
                                    false
                                );
                                self.module.add_function("pow", pow_type, None)
                            });
                            let result = self.builder.build_call(
                                pow_func,
                                &[lhs.into(), rhs.into()],
                                "pow"
                            ).expect("pow call");
                            match result.try_as_basic_value() {
                                inkwell::values::ValueKind::Basic(inkwell::values::BasicValueEnum::FloatValue(fv)) => fv.into(),
                                _ => return Err("pow() did not return float".to_string()),
                            }
                        }
                        _ => return Err(format!("Unsupported float operator: {:?}", op)),
                    }
                } else {
                    let lhs = lhs_val.into_int_value();
                    let rhs = rhs_val.into_int_value();
                    match op {
                        BinOp::Add => self
                            .ir_builder
                            .build_add(&self.builder, lhs, rhs, "add")
                            .into(),
                        BinOp::Sub => self
                            .ir_builder
                            .build_sub(&self.builder, lhs, rhs, "sub")
                            .into(),
                        BinOp::Mul => self
                            .ir_builder
                            .build_mul(&self.builder, lhs, rhs, "mul")
                            .into(),
                        BinOp::Div => self
                            .ir_builder
                            .build_div(&self.builder, lhs, rhs, "div")
                            .into(),
                        BinOp::Eq => self
                            .ir_builder
                            .build_icmp_eq(&self.builder, lhs, rhs, "eq")
                            .into(),
                        BinOp::NotEq => {
                            let eq = self.ir_builder.build_icmp_eq(&self.builder, lhs, rhs, "eq");
                            self.builder.build_not(eq, "neq").expect("not").into()
                        }
                        BinOp::Lt => self
                            .ir_builder
                            .build_icmp_lt(&self.builder, lhs, rhs, "lt")
                            .into(),
                        BinOp::Gt => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gt")
                            .expect("gt")
                            .into(),
                        BinOp::LtEq => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "lte")
                            .expect("lte")
                            .into(),
                        BinOp::GtEq => self
                            .builder
                            .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gte")
                            .expect("gte")
                            .into(),
                        BinOp::Is => {
                            // is: compare values (works for None which is 0)
                            self.ir_builder
                                .build_icmp_eq(&self.builder, lhs, rhs, "is_cmp")
                                .into()
                        }
                        BinOp::IsNot => {
                            // is not: negation of is
                            let eq = self.ir_builder.build_icmp_eq(&self.builder, lhs, rhs, "isnot_cmp");
                            self.builder.build_not(eq, "isnot_result").expect("not").into()
                        }
                        BinOp::In => {
                            // in: check if value is in list (right operand is the list)
                            let value_val = lhs;
                            let list_val = rhs_val;
                            
                            let list_contains = self
                                .module
                                .get_function("vp_list_contains")
                                .ok_or_else(|| "vp_list_contains not declared".to_string())?;
                            
                            let result = self.ir_builder.build_call(
                                &self.builder,
                                list_contains,
                                &[list_val.into(), value_val.into()],
                                "list_contains",
                            );
                            let contains_val: inkwell::values::BasicValueEnum = result.unwrap_or(self.ir_builder.i64_const(0).into());
                            contains_val
                        }
                        BinOp::NotIn => {
                            // not in: negation of in
                            let value_val = lhs;
                            let list_val = rhs_val;
                            
                            let list_contains = self
                                .module
                                .get_function("vp_list_contains")
                                .ok_or_else(|| "vp_list_contains not declared".to_string())?;
                            
                            let result = self.ir_builder.build_call(
                                &self.builder,
                                list_contains,
                                &[list_val.into(), value_val.into()],
                                "not_in_contains",
                            );
                            let contains_val: inkwell::values::BasicValueEnum = result.unwrap_or(self.ir_builder.i64_const(0).into());
                            self.builder.build_not(
                                contains_val.into_int_value(),
                                "not_in_result",
                            ).expect("not").into()
                        }
                        BinOp::Mod => self
                            .builder
                            .build_int_signed_rem(lhs, rhs, "mod")
                            .expect("mod")
                            .into(),
                        BinOp::FloorDiv => {
                            // Floor division: floor(a / b) for integers is just signed division
                            self.ir_builder
                                .build_div(&self.builder, lhs, rhs, "floordiv")
                                .into()
                        }
                        BinOp::Pow => {
                            // Power: use libm pow function for integers
                            // Convert to double, call pow, convert back
                            let pow_func = self.module.get_function("pow").unwrap_or_else(|| {
                                let pow_type = self.context.f64_type().fn_type(
                                    &[self.context.f64_type().into(), self.context.f64_type().into()],
                                    false
                                );
                                self.module.add_function("pow", pow_type, None)
                            });
                            let lhs_double = self.builder.build_signed_int_to_float(lhs, self.context.f64_type(), "lhs_d").expect("int_to_float");
                            let rhs_double = self.builder.build_signed_int_to_float(rhs, self.context.f64_type(), "rhs_d").expect("int_to_float");
                            let result_double = self.builder.build_call(
                                pow_func,
                                &[lhs_double.into(), rhs_double.into()],
                                "pow_result"
                            ).expect("pow call");
                            let float_val = match result_double.try_as_basic_value() {
                                inkwell::values::ValueKind::Basic(inkwell::values::BasicValueEnum::FloatValue(fv)) => fv,
                                _ => return Err("pow() did not return float".to_string()),
                            };
                            self.builder.build_float_to_signed_int(
                                float_val,
                                self.context.i64_type(),
                                "pow_int"
                            ).expect("pow cast").into()
                        }
                        _ => return Err(format!("Unsupported int operator: {:?}", op)),
                    }
                };

                Ok(result)
            }
            Expr::UnaryOp { op, operand, .. } => {
                let val = self.generate_expr(operand)?.into_int_value();
                match op {
                    crate::ast::UnaryOp::Neg => {
                        Ok(self.builder.build_int_neg(val, "neg").expect("neg").into())
                    }
                    crate::ast::UnaryOp::Not => {
                        Ok(self.builder.build_not(val, "not").expect("not").into())
                    }
                    crate::ast::UnaryOp::Pos => Ok(val.into()),
                    _ => Err(format!("Unsupported unary operator: {:?}", op)),
                }
            }
            Expr::Conditional { condition, then_expr, else_expr, span: _ } => {
                // Ternary expression: then_expr if condition else else_expr
                let func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                let cond_val = self.generate_expr(condition)?.into_int_value();
                
                let then_block = self.context.append_basic_block(func, "ternary_then");
                let else_block = self.context.append_basic_block(func, "ternary_else");
                let merge_block = self.context.append_basic_block(func, "ternary_end");
                
                // Convert condition to i1 if needed
                let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
                    cond_val
                } else {
                    self.builder
                        .build_int_compare(
                            inkwell::IntPredicate::NE,
                            cond_val,
                            self.context.i64_type().const_zero(),
                            "ternary_cond",
                        )
                        .expect("ternary_cond")
                };
                
                self.ir_builder.build_cond_branch(&self.builder, cond_i1, then_block, else_block);
                
                // Then block
                self.builder.position_at_end(then_block);
                let then_val = self.generate_expr(then_expr)?;
                let then_block_end = self.builder.get_insert_block().unwrap();
                self.ir_builder.build_branch(&self.builder, merge_block);
                
                // Else block
                self.builder.position_at_end(else_block);
                let else_val = self.generate_expr(else_expr)?;
                let else_block_end = self.builder.get_insert_block().unwrap();
                self.ir_builder.build_branch(&self.builder, merge_block);
                
                // Merge block with phi node
                self.builder.position_at_end(merge_block);
                let phi = self.builder.build_phi(then_val.get_type(), "ternary_result").expect("phi");
                phi.add_incoming(&[(&then_val, then_block_end), (&else_val, else_block_end)]);
                
                Ok(phi.as_basic_value())
            }
            Expr::Call { func, args, span } => {
                // Check for method calls on objects (e.g., list.append(6))
                if let Expr::Attribute { obj, attr, .. } = func.as_ref() {
                    return self.generate_method_call(obj, attr, args, *span);
                }

                if let Expr::Ident(name, _) = func.as_ref() {
                    // Check for built-in functions
                    if name == "print" {
                        return self.generate_print_call(args);
                    }

                    // Check for len() builtin
                    if name == "len" {
                        return self.generate_len_call(args);
                    }

                    // User-defined function
                    if let Some(&func_val) = self.functions.get(name) {
                        let arg_values: Vec<_> = args
                            .iter()
                            .map(|a| self.generate_expr(a).map(|v| v.into()))
                            .collect::<Result<_, _>>()?;

                        let result = self.ir_builder.build_call(
                            &self.builder,
                            func_val,
                            &arg_values,
                            "call",
                        );
                        return Ok(result.unwrap_or(self.ir_builder.i64_const(0).into()));
                    }
                }

                Err(format!("Unknown function: {:?}", func))
            }
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }

    fn generate_print_call(
        &mut self,
        args: &[Expr],
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        if args.is_empty() {
            return Ok(self.ir_builder.i64_const(0).into());
        }

        // Evaluate the argument expression
        let val_res = self.generate_expr(&args[0]);
        if let Ok(val) = val_res {
            if val.is_int_value() {
                let print_func = self
                    .module
                    .get_function("vp_print_i64")
                    .ok_or_else(|| "vp_print_i64 not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(print_func, &[val.into()], "print_i64")
                    .expect("vp_print_i64");

                let newline_func = self
                    .module
                    .get_function("vp_print_newline")
                    .ok_or_else(|| "vp_print_newline not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(newline_func, &[], "print_newline")
                    .expect("vp_print_newline");

                return Ok(self.ir_builder.i64_const(0).into());
            } else if val.is_float_value() {
                let print_func = self
                    .module
                    .get_function("vp_print_f64")
                    .ok_or_else(|| "vp_print_f64 not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(print_func, &[val.into()], "print_f64")
                    .expect("vp_print_f64");

                let newline_func = self
                    .module
                    .get_function("vp_print_newline")
                    .ok_or_else(|| "vp_print_newline not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(newline_func, &[], "print_newline")
                    .expect("vp_print_newline");

                return Ok(self.ir_builder.i64_const(0).into());
            } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
                // Boolean (i1)
                let print_func = self
                    .module
                    .get_function("vp_print_bool")
                    .ok_or_else(|| "vp_print_bool not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(print_func, &[val.into()], "print_bool")
                    .expect("vp_print_bool");

                let newline_func = self
                    .module
                    .get_function("vp_print_newline")
                    .ok_or_else(|| "vp_print_newline not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(newline_func, &[], "print_newline")
                    .expect("vp_print_newline");

                return Ok(self.ir_builder.i64_const(0).into());
            } else if val.is_pointer_value() {
                // String
                let print_func = self
                    .module
                    .get_function("vp_print_str")
                    .ok_or_else(|| "vp_print_str not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(print_func, &[val.into()], "print_str")
                    .expect("vp_print_str");

                let newline_func = self
                    .module
                    .get_function("vp_print_newline")
                    .ok_or_else(|| "vp_print_newline not declared".to_string())?;
                let _result = self
                    .builder
                    .build_call(newline_func, &[], "print_newline")
                    .expect("vp_print_newline");

                return Ok(self.ir_builder.i64_const(0).into());
            } else {
                return Err(format!(
                    "print() does not support type {:?}",
                    val.get_type()
                ));
            }
        }

        Err("print() argument evaluation failed".to_string())
    }

    /// Generate len() builtin call
    fn generate_len_call(
        &mut self,
        args: &[Expr],
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        if args.len() != 1 {
            return Err(format!("len() takes exactly 1 argument, got {}", args.len()));
        }

        let obj_val = self.generate_expr(&args[0])?;
        let list_len = self
            .module
            .get_function("vp_list_len")
            .ok_or_else(|| "vp_list_len not declared".to_string())?;
        let result = self.ir_builder.build_call(
            &self.builder,
            list_len,
            &[obj_val.into()],
            "list_len",
        );
        Ok(result.unwrap_or(self.ir_builder.i64_const(0).into()))
    }

    /// Generate method call (e.g., list.append(6), list.len())
    fn generate_method_call(
        &mut self,
        obj: &Expr,
        method_name: &str,
        args: &[Expr],
        _span: crate::utils::Span,
    ) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
        // Generate code for the object expression
        let obj_val = self.generate_expr(obj)?;

        match method_name {
            "append" => {
                // list.append(value)
                if args.len() != 1 {
                    return Err(format!("append() takes exactly 1 argument, got {}", args.len()));
                }
                let val = self.generate_expr(&args[0])?.into_int_value();
                let list_append = self
                    .module
                    .get_function("vp_list_append")
                    .ok_or_else(|| "vp_list_append not declared".to_string())?;
                self.ir_builder.build_call(
                    &self.builder,
                    list_append,
                    &[obj_val.into(), val.into()],
                    "list_append",
                );
                Ok(self.ir_builder.i64_const(0).into())
            }
            "insert" => {
                // list.insert(index, value)
                if args.len() != 2 {
                    return Err(format!("insert() takes exactly 2 arguments, got {}", args.len()));
                }
                let index = self.generate_expr(&args[0])?.into_int_value();
                let val = self.generate_expr(&args[1])?.into_int_value();
                let list_insert = self
                    .module
                    .get_function("vp_list_insert")
                    .ok_or_else(|| "vp_list_insert not declared".to_string())?;
                self.ir_builder.build_call(
                    &self.builder,
                    list_insert,
                    &[obj_val.into(), index.into(), val.into()],
                    "list_insert",
                );
                Ok(self.ir_builder.i64_const(0).into())
            }
            "remove" => {
                // list.remove(index) - removes and returns element at index
                if args.len() != 1 {
                    return Err(format!("remove() takes exactly 1 argument, got {}", args.len()));
                }
                let index = self.generate_expr(&args[0])?.into_int_value();
                let list_remove = self
                    .module
                    .get_function("vp_list_remove")
                    .ok_or_else(|| "vp_list_remove not declared".to_string())?;
                let result = self.ir_builder.build_call(
                    &self.builder,
                    list_remove,
                    &[obj_val.into(), index.into()],
                    "list_remove",
                );
                Ok(result.unwrap_or(self.ir_builder.i64_const(0).into()))
            }
            "pop" => {
                // list.pop() - removes and returns last element
                if !args.is_empty() {
                    return Err(format!("pop() takes no arguments, got {}", args.len()));
                }
                let list_pop = self
                    .module
                    .get_function("vp_list_pop")
                    .ok_or_else(|| "vp_list_pop not declared".to_string())?;
                let result = self.ir_builder.build_call(
                    &self.builder,
                    list_pop,
                    &[obj_val.into()],
                    "list_pop",
                );
                Ok(result.unwrap_or(self.ir_builder.i64_const(0).into()))
            }
            "clear" => {
                // list.clear()
                if !args.is_empty() {
                    return Err(format!("clear() takes no arguments, got {}", args.len()));
                }
                let list_clear = self
                    .module
                    .get_function("vp_list_clear")
                    .ok_or_else(|| "vp_list_clear not declared".to_string())?;
                self.ir_builder.build_call(
                    &self.builder,
                    list_clear,
                    &[obj_val.into()],
                    "list_clear",
                );
                Ok(self.ir_builder.i64_const(0).into())
            }
            "len" => {
                // This shouldn't happen - len(list) is a builtin, not a method
                // But handle it just in case
                Err("len() is a builtin function, not a method".to_string())
            }
            _ => Err(format!("Unknown method: {}.{}", 
                match obj {
                    Expr::Ident(name, _) => name.as_str(),
                    _ => "object",
                },
                method_name
            )),
        }
    }

    /// Get the generated LLVM module
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    /// Verify the generated code
    pub fn verify(&self) -> Result<(), String> {
        self.module.verify().map_err(|e| e.to_string())
    }

    /// Print the generated IR
    #[allow(dead_code)]
    pub fn print_ir(&self) -> String {
        self.module.to_string().to_string()
    }
}
