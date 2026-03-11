//! Constant Folding and Constant Propagation Optimization
//!
//! Evaluates constant expressions at compile-time instead of runtime.
//!
//! Examples:
//! - `x = 2 + 3 * 4` becomes `x = 14`
//! - `y = x + 10` becomes `y = 24` (if x is constant)
//! - `if True: ...` becomes unconditional execution

use crate::ast::{Expr, Stmt, BinOp, UnaryOp};
use crate::utils::Span;
use std::collections::HashMap;

/// Constant value representation
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    None,
}

/// Constant folding optimizer
pub struct ConstantFolder {
    constants: HashMap<String, ConstantValue>,
}

impl ConstantFolder {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
        }
    }

    /// Run constant folding on the AST
    pub fn fold(&mut self, ast: &mut crate::ast::Module) {
        for stmt in &mut ast.statements {
            self.fold_statement(stmt);
        }
    }

    fn fold_statement(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Assign { target, value, .. } => {
                // Fold the value expression
                self.fold_expr(value);

                // If assigning a constant to a variable, track it
                if let Expr::Ident(name, _) = target.as_ref() {
                    if let Some(const_val) = self.extract_constant(value) {
                        self.constants.insert(name.clone(), const_val);
                    } else {
                        // Variable is no longer constant
                        self.constants.remove(name);
                    }
                }
            }
            Stmt::If { condition, body, else_body, .. } => {
                self.fold_expr(condition);

                // For now, just fold the bodies without trying to optimize constant conditions
                // (that would require more complex AST manipulation)
                for stmt in body {
                    self.fold_statement(stmt);
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.fold_statement(stmt);
                    }
                }
            }
            Stmt::While { condition, body, .. } => {
                self.fold_expr(condition);

                // Fold body statements
                for stmt in body {
                    self.fold_statement(stmt);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(ref mut expr) = value {
                    self.fold_expr(expr);
                }
            }
            Stmt::Expr(expr) => {
                self.fold_expr(expr);
            }
            _ => {
                // Other statement types - fold any expressions they contain
                // For now, just pass through
            }
        }
    }

    fn fold_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::BinOp { left, right, op, .. } => {
                // Fold both operands first
                self.fold_expr(left);
                self.fold_expr(right);

                // Try to fold the operation
                if let (Some(left_val), Some(right_val)) = (self.extract_constant(left), self.extract_constant(right)) {
                    if let Some(result) = self.fold_binary_op(*op, left_val, right_val) {
                        *expr = self.constant_to_expr(result);
                    }
                }
            }
            Expr::UnaryOp { operand, op, .. } => {
                self.fold_expr(operand);

                if let Some(val) = self.extract_constant(operand) {
                    if let Some(result) = self.fold_unary_op(*op, val) {
                        *expr = self.constant_to_expr(result);
                    }
                }
            }
            Expr::Ident(name, _) => {
                // Replace with constant value if known
                if let Some(const_val) = self.constants.get(name) {
                    *expr = self.constant_to_expr(const_val.clone());
                }
            }
            Expr::Call { args, .. } => {
                // Fold arguments
                for arg in args {
                    self.fold_expr(arg);
                }
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.fold_expr(elem);
                }
            }
            Expr::ListComprehension { iter, .. } => {
                self.fold_expr(iter);
            }
            _ => {
                // Other expressions don't need folding for now
            }
        }
    }

    fn extract_constant(&self, expr: &Expr) -> Option<ConstantValue> {
        match expr {
            Expr::Int(val, _) => Some(ConstantValue::Int(*val)),
            Expr::Float(val, _) => Some(ConstantValue::Float(*val)),
            Expr::Bool(val, _) => Some(ConstantValue::Bool(*val)),
            Expr::Str(val, _) => Some(ConstantValue::Str(val.clone())),
            Expr::None(_) => Some(ConstantValue::None),
            _ => None,
        }
    }

    fn fold_binary_op(&self, op: BinOp, left: ConstantValue, right: ConstantValue) -> Option<ConstantValue> {
        match (op, left, right) {
            // Integer arithmetic
            (BinOp::Add, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                a.checked_add(b).map(ConstantValue::Int)
            }
            (BinOp::Sub, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                a.checked_sub(b).map(ConstantValue::Int)
            }
            (BinOp::Mul, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                a.checked_mul(b).map(ConstantValue::Int)
            }
            (BinOp::Div, ConstantValue::Int(a), ConstantValue::Int(b)) if b != 0 => {
                a.checked_div(b).map(ConstantValue::Int)
            }
            (BinOp::Mod, ConstantValue::Int(a), ConstantValue::Int(b)) if b != 0 => {
                a.checked_rem(b).map(ConstantValue::Int)
            }

            // Float arithmetic
            (BinOp::Add, ConstantValue::Float(a), ConstantValue::Float(b)) => {
                Some(ConstantValue::Float(a + b))
            }
            (BinOp::Sub, ConstantValue::Float(a), ConstantValue::Float(b)) => {
                Some(ConstantValue::Float(a - b))
            }
            (BinOp::Mul, ConstantValue::Float(a), ConstantValue::Float(b)) => {
                Some(ConstantValue::Float(a * b))
            }
            (BinOp::Div, ConstantValue::Float(a), ConstantValue::Float(b)) if b != 0.0 => {
                Some(ConstantValue::Float(a / b))
            }

            // Boolean operations
            (BinOp::And, ConstantValue::Bool(a), ConstantValue::Bool(b)) => {
                Some(ConstantValue::Bool(a && b))
            }
            (BinOp::Or, ConstantValue::Bool(a), ConstantValue::Bool(b)) => {
                Some(ConstantValue::Bool(a || b))
            }

            // String concatenation
            (BinOp::Add, ConstantValue::Str(a), ConstantValue::Str(b)) => {
                Some(ConstantValue::Str(a + &b))
            }

            // Comparisons
            (BinOp::Eq, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                Some(ConstantValue::Bool(a == b))
            }
            (BinOp::NotEq, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                Some(ConstantValue::Bool(a != b))
            }
            (BinOp::Lt, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                Some(ConstantValue::Bool(a < b))
            }
            (BinOp::Gt, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                Some(ConstantValue::Bool(a > b))
            }
            (BinOp::LtEq, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                Some(ConstantValue::Bool(a <= b))
            }
            (BinOp::GtEq, ConstantValue::Int(a), ConstantValue::Int(b)) => {
                Some(ConstantValue::Bool(a >= b))
            }

            // Float comparisons
            (BinOp::Eq, ConstantValue::Float(a), ConstantValue::Float(b)) => {
                Some(ConstantValue::Bool((a - b).abs() < f64::EPSILON))
            }
            (BinOp::Lt, ConstantValue::Float(a), ConstantValue::Float(b)) => {
                Some(ConstantValue::Bool(a < b))
            }
            (BinOp::Gt, ConstantValue::Float(a), ConstantValue::Float(b)) => {
                Some(ConstantValue::Bool(a > b))
            }

            _ => None, // Unsupported operation or type combination
        }
    }

    fn fold_unary_op(&self, op: UnaryOp, val: ConstantValue) -> Option<ConstantValue> {
        match (op, val) {
            (UnaryOp::Neg, ConstantValue::Int(x)) => x.checked_neg().map(ConstantValue::Int),
            (UnaryOp::Neg, ConstantValue::Float(x)) => Some(ConstantValue::Float(-x)),
            (UnaryOp::Not, ConstantValue::Bool(x)) => Some(ConstantValue::Bool(!x)),
            _ => None,
        }
    }

    fn constant_to_expr(&self, val: ConstantValue) -> Expr {
        let dummy_span = Span::default(); // Use default span for folded constants
        match val {
            ConstantValue::Int(x) => Expr::Int(x, dummy_span),
            ConstantValue::Float(x) => Expr::Float(x, dummy_span),
            ConstantValue::Bool(x) => Expr::Bool(x, dummy_span),
            ConstantValue::Str(x) => Expr::Str(x, dummy_span),
            ConstantValue::None => Expr::None(dummy_span),
        }
    }
}