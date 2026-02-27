use crate::ast::{BinOp, Expr, Module, SelectCaseKind, Stmt, Type, UnaryOp};
use crate::semantic::symbol_table::{Symbol, SymbolKind, SymbolTable};
use std::collections::HashMap;

/// Type checker for Viper programs
pub struct TypeChecker {
    symbol_table: SymbolTable,
    errors: Vec<TypeError>,
    /// Map from expression to inferred type
    expr_types: HashMap<usize, Type>,
    /// Map from channel variable name to element type (for Chan[T] inference)
    #[allow(dead_code)]
    channel_types: HashMap<String, Type>,
}

/// Type error with location information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: crate::utils::Span,
}

impl TypeError {
    pub fn new(message: String, span: crate::utils::Span) -> Self {
        Self { message, span }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.message, self.span)
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            expr_types: HashMap::new(),
            channel_types: HashMap::new(),
        }
    }

    /// Type check a complete module
    pub fn check(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        self.errors.clear();

        // First pass: collect function declarations
        for stmt in &module.statements {
            if let Stmt::Function { name, params, return_type, span, .. } = stmt {
                let param_types: Vec<Type> =
                    params.iter().map(|p| p.type_ann.clone().unwrap_or(Type::Infer)).collect();

                let symbol = Symbol::new_function(
                    name.clone(),
                    param_types,
                    return_type.clone(),
                    *span,
                    self.symbol_table.current_scope_id(),
                );
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e, *span));
                }
            }
        }

        // Second pass: type check all statements
        // Module-level (scope 0) assignments create immutable constants
        for stmt in &module.statements {
            self.check_stmt_module(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Get the inferred type of an expression
    pub fn get_expr_type(&self, expr: &Expr) -> Option<Type> {
        // Use span as a rough identifier for the expression
        let span = expr.span();
        // Try to find in our type map or infer from the expression itself
        self.expr_types.get(&(span.start as usize)).cloned().or_else(|| self.infer_expr_type(expr))
    }

    /// Infer the type of an expression
    fn infer_expr_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Int(_, _) => Some(Type::I64),
            Expr::BigInt(_, _) => Some(Type::BigInt),
            Expr::Float(_, _) => Some(Type::F64),
            Expr::Bool(_, _) => Some(Type::Bool),
            Expr::Str(_, _) | Expr::FString(_, _) => Some(Type::Str),
            Expr::Bytes(_, _) => Some(Type::Bytes),
            Expr::None(_) => Some(Type::None),
            Expr::Ident(name, _) => self.symbol_table.lookup(name).and_then(|s| s.get_type()),
            Expr::List { elements, .. } => {
                if elements.is_empty() {
                    Some(Type::List(Box::new(Type::Infer)))
                } else {
                    self.infer_expr_type(&elements[0]).map(|t| Type::List(Box::new(t)))
                }
            }
            Expr::Array { elements, size, .. } => {
                if elements.is_empty() {
                    Some(Type::Array(Box::new(Type::Infer), size.unwrap_or(0)))
                } else {
                    self.infer_expr_type(&elements[0])
                        .map(|t| Type::Array(Box::new(t), size.unwrap_or(elements.len())))
                }
            }
            Expr::Tuple { elements, .. } => {
                let types: Vec<Type> =
                    elements.iter().filter_map(|e| self.infer_expr_type(e)).collect();
                if types.len() == elements.len() {
                    Some(Type::Tuple(types))
                } else {
                    None
                }
            }
            Expr::Dict { .. } => Some(Type::Var("dict".to_string())),
            Expr::Await { future, .. } => {
                // Await returns the type of the future
                self.infer_expr_type(future)
            }
            Expr::Call { func, args: _, span: _ } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    // Handle concurrency builtins (Phase 3)
                    match name.as_str() {
                        "chan" => Some(Type::Infer), // Chan element type inferred from usage
                        "recv" => Some(Type::Infer), // Returns channel element type
                        "WaitGroup" => Some(Type::WaitGroup),
                        "send" | "add" | "done" | "wait" => Some(Type::None),
                        _ => {
                            if let Some(symbol) = self.symbol_table.lookup(name) {
                                if let SymbolKind::Function { return_type, .. } = &symbol.kind {
                                    return_type.clone()
                                } else if let SymbolKind::Builtin { signature } = &symbol.kind {
                                    // Handle builtin return types
                                    match signature {
                                        crate::semantic::symbol_table::BuiltinSignature::Print => {
                                            Some(Type::None)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::Range => {
                                            Some(Type::List(Box::new(Type::I64)))
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::Len => {
                                            Some(Type::I64)
                                        }
                                        _ => Some(Type::Infer),
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            }
            Expr::BinOp { op, left, right, .. } => match op {
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Mod
                | BinOp::FloorDiv
                | BinOp::Pow => self.infer_expr_type(left).or_else(|| self.infer_expr_type(right)),
                BinOp::Eq
                | BinOp::NotEq
                | BinOp::Lt
                | BinOp::LtEq
                | BinOp::Gt
                | BinOp::GtEq
                | BinOp::And
                | BinOp::Or => Some(Type::Bool),
                BinOp::NullCoalesce => self.infer_expr_type(left).or_else(|| self.infer_expr_type(right)),
                _ => Some(Type::I64),
            },
            Expr::UnaryOp { op, operand, .. } => match op {
                UnaryOp::Neg | UnaryOp::Pos => self.infer_expr_type(operand),
                UnaryOp::Not => Some(Type::Bool),
                _ => Some(Type::I64),
            },
            Expr::Index { obj, .. } => {
                if let Some(obj_type) = self.infer_expr_type(obj) {
                    match obj_type {
                        Type::List(elem_type) => Some(*elem_type),
                        Type::Var(s) if s.starts_with("dict") => Some(Type::Infer),
                        _ => Some(Type::I64),
                    }
                } else {
                    None
                }
            }
            Expr::Attribute { .. } => Some(Type::Infer),
            Expr::Conditional { then_expr, .. } => self.infer_expr_type(then_expr),
            Expr::Lambda { .. } => Some(Type::Var("lambda".to_string())),
            Expr::ListComprehension { .. } => Some(Type::List(Box::new(Type::Infer))),
            Expr::Slice { .. } => Some(Type::List(Box::new(Type::Infer))),
        }
    }

    /// Check a statement at module level (assignments create immutable constants)
    fn check_stmt_module(&mut self, stmt: &Stmt) {
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
    fn check_stmt(&mut self, stmt: &Stmt) {
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
            } => {
                // Enter function scope
                self.symbol_table.enter_scope();

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
            _ => {
                // TODO: Handle other statement types
            }
        }
    }

    /// Check an expression and return its type
    fn check_expr(&mut self, expr: &Expr) -> Option<Type> {
        let expr_type = self.infer_expr_type(expr);

        // Store the inferred type
        let span = expr.span();
        self.expr_types.insert(span.start as usize, expr_type.clone().unwrap_or(Type::Infer));

        match expr {
            Expr::BinOp { left, right, op, span } => {
                let left_type = self.check_expr(left);
                let right_type = self.check_expr(right);

                if let (Some(lt), Some(rt)) = (left_type, right_type) {
                    match op {
                        BinOp::Add => {
                            // List concatenation: List + List is allowed
                            match (&lt, &rt) {
                                (Type::List(_), Type::List(_)) => {
                                    // List concatenation is valid
                                }
                                _ => {
                                    // For other types, require numeric
                                    if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                        self.errors.push(TypeError::new(
                                            format!(
                                                "Arithmetic operators require numeric types, got {} and {}",
                                                lt, rt
                                            ),
                                            *span,
                                        ));
                                    }
                                }
                            }
                        }
                        BinOp::Mul => {
                            // List repetition: List * int or int * List is allowed
                            let is_list_repeat = match (&lt, &rt) {
                                (Type::List(_), Type::I64) => true,
                                (Type::I64, Type::List(_)) => true,
                                _ => false,
                            };
                            if !is_list_repeat {
                                // For other types, require numeric
                                if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                    self.errors.push(TypeError::new(
                                        format!(
                                            "Arithmetic operators require numeric types, got {} and {}",
                                            lt, rt
                                        ),
                                        *span,
                                    ));
                                }
                            }
                        }
                        BinOp::Sub | BinOp::Div | BinOp::Mod | BinOp::FloorDiv | BinOp::Pow => {
                            if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Arithmetic operators require numeric types, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        BinOp::Eq | BinOp::NotEq => {
                            if lt != rt {
                                self.errors.push(TypeError::new(
                                    format!("Cannot compare {} with {}", lt, rt),
                                    *span,
                                ));
                            }
                        }
                        BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                            if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Comparison operators require numeric types, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        BinOp::And | BinOp::Or => {
                            if lt != Type::Bool || rt != Type::Bool {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Logical operators require bool, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Expr::Call { func, args, span } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    if let Some(symbol) = self.symbol_table.lookup(name) {
                        if let SymbolKind::Function { params, .. } = &symbol.kind {
                            if params.len() != args.len() {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Expected {} arguments, got {}",
                                        params.len(),
                                        args.len()
                                    ),
                                    *span,
                                ));
                            }
                        }
                    }
                }

                // Check argument types
                for arg in args {
                    self.check_expr(arg);
                }
            }
            Expr::Index { obj, index, span } => {
                self.check_expr(obj);
                let index_type = self.check_expr(index);

                if let Some(it) = index_type {
                    if it != Type::I64 {
                        self.errors
                            .push(TypeError::new(format!("Index must be i64, got {}", it), *span));
                    }
                }
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            Expr::Array { elements, .. } => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            Expr::Dict { pairs, span: _ } => {
                for (key, value) in pairs {
                    self.check_expr(key);
                    self.check_expr(value);

                    // Check that key type is hashable
                    if let Some(key_type) = self.get_expr_type(key) {
                        if !key_type.is_fully_hashable() {
                            self.errors.push(TypeError::new(
                                format!("Dictionary keys must be hashable, got {}", key_type),
                                key.span(),
                            ));
                        }
                    }
                }
            }
            Expr::Conditional { condition, then_expr, else_expr, span } => {
                let cond_type = self.check_expr(condition);
                if let Some(t) = cond_type {
                    if t != Type::Bool {
                        self.errors.push(TypeError::new(
                            format!("Condition must be bool, got {}", t),
                            *span,
                        ));
                    }
                }
                self.check_expr(then_expr);
                self.check_expr(else_expr);
            }
            _ => {}
        }

        expr_type
    }

    /// Check that all return statements are consistent with declared return type
    fn check_return_consistency(&mut self, body: &[Stmt], return_type: &Option<Type>) {
        // This is a simplified check - a full implementation would track
        // all code paths and ensure they all return the correct type
        for stmt in body {
            if let Stmt::Return { value, span } = stmt {
                match (return_type, value) {
                    (Some(rt), Some(v)) => {
                        let v_type = self.check_expr(v);
                        if let Some(vt) = v_type {
                            if !self.is_compatible(rt, &vt) {
                                self.errors.push(TypeError::new(
                                    format!("Cannot return {} as {}", vt, rt),
                                    *span,
                                ));
                            }
                        }
                    }
                    (Some(rt), None) => {
                        if rt != &Type::None {
                            self.errors.push(TypeError::new(
                                format!("Expected return value of type {}", rt),
                                *span,
                            ));
                        }
                    }
                    (None, Some(v)) => {
                        self.check_expr(v);
                    }
                    (None, None) => {}
                }
            }
        }
    }

    /// Check if a type is numeric
    fn is_numeric(&self, t: &Type) -> bool {
        let resolved = self.symbol_table.resolve_type_alias(t);
        resolved.is_numeric()
    }

    /// Check if two types are compatible
    fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        // Resolve type aliases first
        let expected_resolved = self.symbol_table.resolve_type_alias(expected);
        let actual_resolved = self.symbol_table.resolve_type_alias(actual);

        match (&expected_resolved, &actual_resolved) {
            (Type::Infer, _) | (_, Type::Infer) => true,
            (Type::Error, _) | (_, Type::Error) => true,
            (e, a) if e.is_integer() && a.is_integer() => true,
            (e, a) if e.is_float() && a.is_float() => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Str, Type::Str) => true,
            (Type::None, Type::None) => true,
            (Type::List(a), Type::List(b)) => self.is_compatible(a, b),
            (Type::Array(a1, s1), Type::Array(a2, s2)) => s1 == s2 && self.is_compatible(a1, a2),
            // Channel types: Chan[T] is compatible with Chan[U] if T is compatible with U
            (Type::Chan(elem_expected), Type::Chan(elem_actual)) => {
                self.is_compatible(elem_expected, elem_actual)
            }
            // WaitGroup is only compatible with itself
            (Type::WaitGroup, Type::WaitGroup) => true,
            // Optional types: T? is compatible with T (and vice versa for assignments)
            (Type::Optional(inner_expected), Type::Optional(inner_actual)) => {
                self.is_compatible(inner_expected, inner_actual)
            }
            // Non-optional can be assigned to Optional (Some value)
            (Type::Optional(inner), _) => self.is_compatible(inner, &actual_resolved),
            // Tuple types
            (Type::Tuple(expected_types), Type::Tuple(actual_types)) => {
                if expected_types.len() != actual_types.len() {
                    return false;
                }
                expected_types
                    .iter()
                    .zip(actual_types.iter())
                    .all(|(e, a)| self.is_compatible(e, a))
            }
            _ => false,
        }
    }

    /// Get collected errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Get the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
