use crate::ast::{Expr, SelectCaseKind, Stmt, Type};
use crate::semantic::symbol_table::{Symbol, SymbolKind};
use crate::semantic::type_checker::{TypeChecker, TypeError};

impl TypeChecker {
    /// Collect instance fields from __init__ method body
    fn collect_instance_fields(body: &[Stmt], fields: &mut Vec<(String, Type)>) {
        for stmt in body {
            match stmt {
                Stmt::Assign { target, value, .. } => {
                    if let Expr::Attribute { obj, attr, .. } = target.as_ref() {
                        if let Expr::Ident(obj_name, _) = obj.as_ref() {
                            if obj_name == "self" {
                                // Infer field type from the assigned value
                                let field_type = match value.as_ref() {
                                    Expr::Int(_, _) => Type::I64,
                                    Expr::Float(_, _) => Type::F64,
                                    Expr::Bool(_, _) => Type::Bool,
                                    Expr::Str(_, _) | Expr::FString(_, _) => Type::Str,
                                    Expr::BigInt(_, _) => Type::BigInt,
                                    Expr::List { .. } => Type::List(Box::new(Type::Infer)),
                                    Expr::None(_) => Type::None,
                                    _ => Type::Infer,
                                };
                                
                                // Add field if not already present
                                if !fields.iter().any(|(name, _)| name == attr) {
                                    fields.push((attr.clone(), field_type));
                                }
                            }
                        }
                    }
                }
                Stmt::If { body: if_body, elif_blocks, else_body, .. } => {
                    Self::collect_instance_fields(if_body, fields);
                    for (_, elif_body) in elif_blocks {
                        Self::collect_instance_fields(elif_body, fields);
                    }
                    if let Some(else_body) = else_body {
                        Self::collect_instance_fields(else_body, fields);
                    }
                }
                Stmt::While { body: while_body, .. } => {
                    Self::collect_instance_fields(while_body, fields);
                }
                Stmt::For { body: for_body, .. } => {
                    Self::collect_instance_fields(for_body, fields);
                }
                _ => {}
            }
        }
    }

    /// Check a statement at module level (assignments create immutable constants)
    pub(crate) fn check_stmt_module(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { target, value, span } => {
                let value_type = self.check_expr(value);

                if let Expr::Ident(name, _) = target.as_ref() {
                    // Module-level assignments create immutable constants
                    let kind =
                        SymbolKind::Variable { mutable: false, type_ann: value_type.clone() };
                    let symbol = Symbol::new(
                        name.clone(),
                        kind,
                        *span,
                        self.symbol_table.current_scope_id(),
                    );
                    if let Err(e) = self.symbol_table.insert(symbol) {
                        self.errors.push(TypeError::new(e, *span));
                    }
                } else {
                    // Non-identifier target, use regular check
                    self.check_stmt(stmt);
                }
            }
            _ => {
                // Other statements use regular checking
                self.check_stmt(stmt);
            }
        }
    }

    /// Check a statement
    pub(crate) fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
            Stmt::Assign { target, value, span } => {
                let value_type = self.check_expr(value);
                let target_type = self.check_expr(target);

                if let Expr::Ident(name, _) = target.as_ref() {
                    // Check if variable exists and is mutable
                    if let Some(symbol) = self.symbol_table.lookup(name) {
                        if let SymbolKind::Variable { mutable, .. } = &symbol.kind {
                            if !*mutable {
                                self.errors.push(TypeError::new(
                                    format!("Cannot assign to immutable variable '{}'", name),
                                    *span,
                                ));
                            }
                        }
                    } else {
                        // Variable doesn't exist - create it with inferred type (implicit declaration)
                        let kind = SymbolKind::Variable { mutable: true, type_ann: value_type.clone() };
                        let symbol = Symbol::new(
                            name.clone(),
                            kind,
                            *span,
                            self.symbol_table.current_scope_id(),
                        );
                        if let Err(e) = self.symbol_table.insert(symbol) {
                            self.errors.push(TypeError::new(e, *span));
                        }
                    }
                }

                if let (Some(tt), Some(vt)) = (target_type, value_type) {
                    if !self.is_compatible(&tt, &vt) {
                        self.errors
                            .push(TypeError::new(format!("Cannot assign {} to {}", vt, tt), *span));
                    } else {
                        // Add bounds checking for integers
                        if let Expr::Int(n, _) = value.as_ref() {
                            let out_of_bounds = match tt {
                                Type::I8 => n < &(i8::MIN as i64) || n > &(i8::MAX as i64),
                                Type::I16 => n < &(i16::MIN as i64) || n > &(i16::MAX as i64),
                                Type::I32 => n < &(i32::MIN as i64) || n > &(i32::MAX as i64),
                                _ => false,
                            };
                            if out_of_bounds {
                                self.errors.push(TypeError::new(
                                    format!("Value {} is out of bounds for type {}", n, tt),
                                    value.span(),
                                ));
                            }
                        }
                    }
                }
            }
            Stmt::Declare { name, type_ann, value, mutable, span } => {
                if let Some(val) = value {
                    let value_type = self.check_expr(val);

                    // Check type compatibility
                    if let Some(ann_type) = type_ann {
                        if let Some(vt) = &value_type {
                            if !self.is_compatible(ann_type, vt) {
                                self.errors.push(TypeError::new(
                                    format!("Cannot assign {} to {}", vt, ann_type),
                                    *span,
                                ));
                            } else {
                                // Add bounds checking for integers
                                if let Expr::Int(n, _) = val {
                                    let out_of_bounds = match ann_type {
                                        Type::I8 => *n < i8::MIN as i64 || *n > i8::MAX as i64,
                                        Type::I16 => *n < i16::MIN as i64 || *n > i16::MAX as i64,
                                        Type::I32 => *n < i32::MIN as i64 || *n > i32::MAX as i64,
                                        _ => false,
                                    };
                                    if out_of_bounds {
                                        self.errors.push(TypeError::new(
                                            format!(
                                                "Value {} is out of bounds for type {}",
                                                n, ann_type
                                            ),
                                            val.span(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                // Insert variable into symbol table
                let kind = SymbolKind::Variable { mutable: *mutable, type_ann: type_ann.clone() };
                let symbol =
                    Symbol::new(name.clone(), kind, *span, self.symbol_table.current_scope_id());
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e, *span));
                }
            }
            Stmt::Global { names, span } => {
                // Global keyword - variables should exist at module level
                // For now, we just register them as mutable variables
                for name in names {
                    let kind = SymbolKind::Variable { mutable: true, type_ann: None };
                    let symbol = Symbol::new(
                        name.clone(),
                        kind,
                        *span,
                        self.symbol_table.current_scope_id(),
                    );
                    // Don't error if already exists (global can be repeated)
                    let _ = self.symbol_table.insert(symbol);
                }
            }
            Stmt::Const { name, value, span } => {
                // Constants must have a value and are immutable
                let value_type = self.check_expr(value);

                // Insert into symbol table as immutable constant
                let kind = SymbolKind::Variable { mutable: false, type_ann: value_type };
                let symbol =
                    Symbol::new(name.clone(), kind, *span, self.symbol_table.current_scope_id());
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e, *span));
                }
            }
            Stmt::If { condition, body, elif_blocks, else_body, span } => {
                let cond_type = self.check_expr(condition);
                if let Some(t) = cond_type {
                    if t != Type::Bool {
                        self.errors.push(TypeError::new(
                            format!("Condition must be bool, got {}", t),
                            *span,
                        ));
                    }
                }

                self.symbol_table.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.symbol_table.exit_scope();

                for (elif_cond, elif_body) in elif_blocks {
                    let elif_type = self.check_expr(elif_cond);
                    if let Some(t) = elif_type {
                        if t != Type::Bool {
                            self.errors.push(TypeError::new(
                                format!("Condition must be bool, got {}", t),
                                elif_cond.span(),
                            ));
                        }
                    }
                    self.symbol_table.enter_scope();
                    for s in elif_body {
                        self.check_stmt(s);
                    }
                    self.symbol_table.exit_scope();
                }

                if let Some(else_body) = else_body {
                    self.symbol_table.enter_scope();
                    for s in else_body {
                        self.check_stmt(s);
                    }
                    self.symbol_table.exit_scope();
                }
            }
            Stmt::While { condition, body, span, .. } => {
                let cond_type = self.check_expr(condition);
                if let Some(t) = cond_type {
                    if t != Type::Bool {
                        self.errors.push(TypeError::new(
                            format!("Condition must be bool, got {}", t),
                            *span,
                        ));
                    }
                }

                self.symbol_table.enter_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::For { target, iter, body, span, .. } => {
                self.check_expr(iter);

                // Bind loop variable
                if let Expr::Ident(name, _) = target.as_ref() {
                    let kind = SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) };
                    let symbol = Symbol::new(
                        name.clone(),
                        kind,
                        target.span(),
                        self.symbol_table.current_scope_id(),
                    );
                    if let Err(e) = self.symbol_table.insert(symbol) {
                        self.errors.push(TypeError::new(e, *span));
                    }
                }

                self.symbol_table.enter_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::Function {
                name: _,
                type_params: _,
                params,
                return_type,
                body,
                span: _,
                is_async: _,
                decorators: _,
            } => {
                // Enter function scope
                self.symbol_table.enter_scope();

                // Save current return type and set new one
                let old_return_type = self.current_return_type.clone();
                self.current_return_type = return_type.clone();

                // Add parameters to scope
                for param in params {
                    let kind = SymbolKind::Parameter { type_ann: param.type_ann.clone() };
                    let symbol = Symbol::new(
                        param.name.clone(),
                        kind,
                        param.span,
                        self.symbol_table.current_scope_id(),
                    );
                    if let Err(e) = self.symbol_table.insert(symbol) {
                        self.errors.push(TypeError::new(e, param.span));
                    }
                }

                // Check function body
                for stmt in body {
                    self.check_stmt(stmt);
                }

                // Check return type consistency
                self.check_return_consistency(body, return_type);

                // Restore previous return type
                self.current_return_type = old_return_type;

                self.symbol_table.exit_scope();
            }
            Stmt::Return { value, span: _ } => {
                if let Some(val) = value {
                    self.check_expr(val);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Pass(_) => {
                // No type checking needed
            }
            Stmt::AugAssign { target, op: _, value, span: _ } => {
                self.check_expr(target);
                self.check_expr(value);
            }
            // Concurrency statements (Phase 3)
            Stmt::Sync { body, .. } => {
                self.symbol_table.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::Task { call, .. } => {
                self.check_expr(call);
            }
            Stmt::Chan { size, span: _ } => {
                self.check_expr(size);
                // Channel creation - type will be inferred from send/recv usage
                // For now, mark as Chan[Infer]
            }
            Stmt::Send { chan, value, span: _ } => {
                let _chan_type = self.check_expr(chan);
                let _value_type = self.check_expr(value);
                // Type checking for send is deferred to runtime
            }
            Stmt::Recv { chan, span: _ } => {
                let _chan_type = self.check_expr(chan);
                // recv() returns a pointer value (type inferred at runtime)
            }
            Stmt::WaitGroup { span: _ } => {
                // WaitGroup creation - returns WaitGroup type (pointer type)
            }
            Stmt::WgAdd { wg, n, span } => {
                let wg_type = self.check_expr(wg);
                let n_type = self.check_expr(n);

                // Type check: wg must be WaitGroup, n must be i64
                if let Some(wt) = wg_type {
                    if wt != Type::WaitGroup {
                        self.errors.push(TypeError::new(
                            format!("add() expects WaitGroup, got {}", wt),
                            *span,
                        ));
                    }
                }
                if let Some(nt) = n_type {
                    if nt != Type::I64 {
                        self.errors.push(TypeError::new(
                            format!("add() expects i64 count, got {}", nt),
                            *span,
                        ));
                    }
                }
            }
            Stmt::WgDone { wg, span } => {
                let wg_type = self.check_expr(wg);

                // Type check: wg must be WaitGroup
                if let Some(wt) = wg_type {
                    if wt != Type::WaitGroup {
                        self.errors.push(TypeError::new(
                            format!("done() expects WaitGroup, got {}", wt),
                            *span,
                        ));
                    }
                }
            }
            Stmt::WgWait { wg, span } => {
                let wg_type = self.check_expr(wg);

                // Type check: wg must be WaitGroup
                if let Some(wt) = wg_type {
                    if wt != Type::WaitGroup {
                        self.errors.push(TypeError::new(
                            format!("wait() expects WaitGroup, got {}", wt),
                            *span,
                        ));
                    }
                }
            }
            Stmt::TypeAlias { name, type_def, span } => {
                // Insert type alias into symbol table
                let kind = SymbolKind::TypeAlias { type_def: type_def.clone() };
                let symbol =
                    Symbol::new(name.clone(), kind, *span, self.symbol_table.current_scope_id());
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e, *span));
                }
            }
            Stmt::Match { subject, cases, span } => {
                self.check_expr(subject);

                for case in cases {
                    if let Some(guard) = &case.guard {
                        let guard_type = self.check_expr(guard);
                        if let Some(gt) = guard_type {
                            if gt != Type::Bool {
                                self.errors.push(TypeError::new(
                                    format!("Match guard must be bool, got {}", gt),
                                    guard.span(),
                                ));
                            }
                        }
                    }

                    self.symbol_table.enter_scope();
                    for stmt in &case.body {
                        self.check_stmt(stmt);
                    }
                    self.symbol_table.exit_scope();
                }

                if cases.is_empty() {
                    self.errors.push(TypeError::new(
                        "Match statement must have at least one case".to_string(),
                        *span,
                    ));
                }
            }
            Stmt::Select { cases, span } => {
                for case in cases {
                    match &case.kind {
                        SelectCaseKind::Recv { chan, var: _ } => {
                            let _ = self.check_expr(chan);
                        }
                        SelectCaseKind::Send { chan, value } => {
                            let _ = self.check_expr(chan);
                            let _ = self.check_expr(value);
                        }
                        SelectCaseKind::Default => {}
                    }

                    self.symbol_table.enter_scope();
                    for stmt in &case.body {
                        self.check_stmt(stmt);
                    }
                    self.symbol_table.exit_scope();
                }

                if cases.is_empty() {
                    self.errors.push(TypeError::new(
                        "Select statement must have at least one case".to_string(),
                        *span,
                    ));
                }
            }
            // New Python keyword statements
            Stmt::Assert { condition, message, span: _ } => {
                let cond_type = self.check_expr(condition);
                // Condition should ideally be boolean
                if let Some(t) = cond_type {
                    if !matches!(t, Type::Bool) {
                        // Assertion condition should ideally be bool, but we allow flexibility
                    }
                }
                if let Some(msg) = message {
                    self.check_expr(msg);
                }
            }
            Stmt::Delete { targets, span: _ } => {
                for target in targets {
                    self.check_expr(target);
                }
            }
            Stmt::Raise { exception, cause, span: _ } => {
                if let Some(exc) = exception {
                    self.check_expr(exc);
                }
                if let Some(c) = cause {
                    self.check_expr(c);
                }
            }
            Stmt::With { items, body, is_async: _, span: _ } => {
                self.symbol_table.enter_scope();
                for item in items {
                    self.check_expr(&item.context_expr);
                    // The optional_vars is bound to the result of __enter__
                    if let Some(var_name) = &item.optional_vars {
                        // Add the variable to the symbol table
                        // Type will be inferred from usage
                        let kind = crate::semantic::symbol_table::SymbolKind::Variable {
                            mutable: true,
                            type_ann: None,
                        };
                        let symbol = crate::semantic::symbol_table::Symbol::new(
                            var_name.clone(),
                            kind,
                            item.span,
                            self.symbol_table.current_scope_id(),
                        );
                        let _ = self.symbol_table.insert(symbol);
                    }
                }
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::Yield { value, span: _ } => {
                if let Some(val) = value {
                    self.check_expr(val);
                }
                // TODO: Track that we're in a generator function
            }
            Stmt::Class { name, bases, body, span, decorators: _, fields, methods } => {
                // Check base classes first (in current scope)
                let mut base_types = Vec::new();
                for base in bases {
                    let base_type = self.check_expr(base);
                    if let Some(t) = base_type {
                        base_types.push(t);
                    }
                }

                // Collect instance fields from __init__ assignments
                let mut instance_fields = fields.iter()
                    .map(|(n, t, _)| (n.clone(), t.clone().unwrap_or(Type::Infer)))
                    .collect::<Vec<_>>();
                
                // Scan __init__ for self.field assignments to discover instance fields
                for stmt in body {
                    if let Stmt::Function { name: method_name, body: method_body, .. } = stmt {
                        if method_name == "__init__" {
                            Self::collect_instance_fields(method_body, &mut instance_fields);
                        }
                    }
                }

                // Register the class in the current (parent) scope BEFORE entering class scope
                let class_symbol = crate::semantic::symbol_table::Symbol::new(
                    name.clone(),
                    SymbolKind::Class {
                        name: name.clone(),
                        bases: base_types.clone(),
                        fields: instance_fields,
                        methods: methods.clone(),
                        method_mangles: Vec::new(),
                    },
                    *span,
                    self.symbol_table.current_scope_id(),
                );
                if let Err(e) = self.symbol_table.insert(class_symbol) {
                    self.errors.push(TypeError::new(e, *span));
                }

                // Now enter class scope for methods and fields
                self.symbol_table.enter_scope();

                // Check methods and fields in class body
                for stmt in body {
                    self.check_stmt(stmt);
                }

                // Exit class scope
                self.symbol_table.exit_scope();
            }
            _ => {
                // TODO: Handle other statement types
            }
        }
    }
}
