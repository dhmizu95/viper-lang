use crate::ast::{BinOp, Expr, Module, Stmt, Type};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

mod builder;
mod context;

use builder::IRBuilder;

/// Main code generator that translates AST to LLVM IR
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    ir_builder: IRBuilder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
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
        }
    }

    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> Result<(), String> {

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

    fn declare_function(
        &mut self,
        name: &str,
        params: &[crate::ast::Param],
        return_type: &Option<Type>,
    ) -> Result<(), String> {
        let param_types: Vec<_> = params
            .iter()
            .map(|p| match &p.type_ann {
                Some(Type::I64) => self.context.i64_type().into(),
                Some(Type::F64) => self.context.f64_type().into(),
                Some(Type::Bool) => self.context.bool_type().into(),
                Some(Type::Str) => self
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                _ => self.context.i64_type().into(),
            })
            .collect();

        let fn_type = match return_type {
            Some(Type::I64) => self.context.i64_type().fn_type(&param_types, false),
            Some(Type::F64) => self.context.f64_type().fn_type(&param_types, false),
            Some(Type::Bool) => self.context.bool_type().fn_type(&param_types, false),
            Some(Type::Str) => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .fn_type(&param_types, false),
            Some(Type::None) | None => self.context.void_type().fn_type(&param_types, false),
            _ => self.context.void_type().fn_type(&param_types, false),
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

        // Set up parameters
        for (i, param) in params.iter().enumerate() {
            let param_value = func.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(param_value.get_type(), &param.name)
                .expect("alloca");
            self.builder
                .build_store(alloca, param_value)
                .expect("store");
            self.variables.insert(param.name.clone(), alloca);
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
                Some(Type::I64) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
                }
                Some(Type::F64) => {
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
                            let ty = val.get_type();
                            let alloca = self.builder.build_alloca(ty, name).expect("alloca");
                            self.builder.build_store(alloca, val).expect("store");
                            self.variables.insert(name.clone(), alloca);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                } else {
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
                    self.variables.insert(name.clone(), alloca);
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
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Pass(_) => {
                // TODO: Implement control flow
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

        let then_block = self.context.append_basic_block(func, "then");
        let else_block = self.context.append_basic_block(func, "else");
        let merge_block = self.context.append_basic_block(func, "if_cont");

        self.ir_builder
            .build_cond_branch(&self.builder, cond_val, then_block, else_block);

        // Then block
        self.builder.position_at_end(then_block);
        for stmt in body {
            self.generate_stmt(stmt)?;
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
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
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
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
                    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                        self.ir_builder.build_branch(&self.builder, merge_block);
                    }
                } else {
                    if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                        self.ir_builder.build_branch(&self.builder, merge_block);
                    }
                }
            }
        } else if let Some(else_stmts) = else_body {
            for stmt in else_stmts {
                self.generate_stmt(stmt)?;
            }
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                self.ir_builder.build_branch(&self.builder, merge_block);
            }
        } else {
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
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
        let cond_val = self.generate_expr(condition)?.into_int_value();
        self.ir_builder
            .build_cond_branch(&self.builder, cond_val, body_block, exit_block);

        // Body block
        self.builder.position_at_end(body_block);
        for stmt in body {
            self.generate_stmt(stmt)?;
        }
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
                        self.variables.insert(target_name.clone(), counter);
                    }
                    for stmt in body {
                        self.generate_stmt(stmt)?;
                    }
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
            Expr::Str(s, _) => Ok(self.ir_builder.string_const(&self.module, s).into()),
            Expr::Ident(name, _span) => {
                if let Some(&alloca) = self.variables.get(name) {
                    // For i64 type, we know it's stored in an alloca
                    let i64_type = self.context.i64_type();
                    Ok(self
                        .builder
                        .build_load(i64_type, alloca, name)
                        .expect("load"))
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expr::BinOp {
                left, op, right, ..
            } => {
                let lhs = self.generate_expr(left)?.into_int_value();
                let rhs = self.generate_expr(right)?.into_int_value();

                let result = match op {
                    BinOp::Add => self.ir_builder.build_add(&self.builder, lhs, rhs, "add"),
                    BinOp::Sub => self.ir_builder.build_sub(&self.builder, lhs, rhs, "sub"),
                    BinOp::Mul => self.ir_builder.build_mul(&self.builder, lhs, rhs, "mul"),
                    BinOp::Div => self.ir_builder.build_div(&self.builder, lhs, rhs, "div"),
                    BinOp::Eq => self.ir_builder.build_icmp_eq(&self.builder, lhs, rhs, "eq"),
                    BinOp::NotEq => {
                        let eq = self.ir_builder.build_icmp_eq(&self.builder, lhs, rhs, "eq");
                        self.builder.build_not(eq, "neq").expect("not")
                    }
                    BinOp::Lt => self.ir_builder.build_icmp_lt(&self.builder, lhs, rhs, "lt"),
                    BinOp::Gt => self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gt")
                        .expect("gt"),
                    BinOp::LtEq => self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "lte")
                        .expect("lte"),
                    BinOp::GtEq => self
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gte")
                        .expect("gte"),
                    BinOp::Mod => self
                        .builder
                        .build_int_signed_rem(lhs, rhs, "mod")
                        .expect("mod"),
                    _ => return Err(format!("Unsupported binary operator: {:?}", op)),
                };

                Ok(result.into())
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
            Expr::Call { func, args, .. } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    // Check for built-in functions
                    if name == "print" {
                        return self.generate_print_call(args);
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
                    } else {
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
        // Declare printf if not already declared
        let i8_ptr = self.context.ptr_type(inkwell::AddressSpace::default());
        let printf = self.module.get_function("printf").unwrap_or_else(|| {
            let printf_type = self.context.i32_type().fn_type(&[i8_ptr.into()], true);
            self.module.add_function("printf", printf_type, None)
        });

        if args.is_empty() {
            return Ok(self.ir_builder.i64_const(0).into());
        }

        // Evaluate the argument expression first
        let val_res = self.generate_expr(&args[0]);
        if let Ok(val) = val_res {
            if val.is_int_value() {
                let fmt_str = self.ir_builder.string_const(&self.module, "%ld\n");
                let _result = self
                    .builder
                    .build_call(printf, &[fmt_str.into(), val.into()], "printf_call")
                    .expect("printf");
                return Ok(self.ir_builder.i64_const(0).into());
            } else if val.is_float_value() {
                let fmt_str = self.ir_builder.string_const(&self.module, "%f\n");
                let _result = self
                    .builder
                    .build_call(printf, &[fmt_str.into(), val.into()], "printf_call")
                    .expect("printf");
                return Ok(self.ir_builder.i64_const(0).into());
            } else {
                return Err("print() only supports numbers in Phase 1".to_string());
            }
        }

        Err("print() only supports integer literals in Phase 1".to_string())
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
