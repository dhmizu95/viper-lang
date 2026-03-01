//! Hindley-Milner Style Type Inference with Constraint Generation and Unification
//!
//! This module implements a constraint-based type inference system:
//! 1. Generate constraints from expressions
//! 2. Unify constraints to solve for type variables
//! 3. Apply substitutions to infer concrete types

use crate::ast::{Expr, Type, UnaryOp, BinOp};
use crate::semantic::symbol_table::{Symbol, SymbolKind};
use crate::semantic::type_checker::{TypeChecker, TypeError};
use std::collections::HashMap;

/// A type constraint representing an equality between two types
#[derive(Debug, Clone)]
pub struct Constraint {
    pub ty1: Type,
    pub ty2: Type,
    pub span: crate::utils::Span,
}

impl Constraint {
    pub fn new(ty1: Type, ty2: Type, span: crate::utils::Span) -> Self {
        Self { ty1, ty2, span }
    }
}

/// A trait constraint representing a type bound (e.g., T: Hashable)
#[derive(Debug, Clone)]
pub struct TraitConstraint {
    /// The type that must satisfy the trait
    pub ty: Type,
    /// The trait name (e.g., "Hashable")
    pub trait_name: String,
    pub span: crate::utils::Span,
}

impl TraitConstraint {
    pub fn new(ty: Type, trait_name: String, span: crate::utils::Span) -> Self {
        Self { ty, trait_name, span }
    }
}

/// A substitution mapping type variables to concrete types
pub type Substitution = HashMap<String, Type>;

impl TypeChecker {
    /// Check if a type satisfies a trait bound
    pub fn check_trait_constraint(&self, ty: &Type, trait_name: &str) -> bool {
        match trait_name {
            "Hashable" => self.is_hashable_type(ty),
            "Comparable" => self.is_comparable_type(ty),
            "Numeric" => ty.is_numeric(),
            _ => false,  // Unknown trait
        }
    }
    
    /// Check if a type is hashable
    fn is_hashable_type(&self, ty: &Type) -> bool {
        ty.is_hashable()
    }
    
    /// Check if a type is comparable
    fn is_comparable_type(&self, ty: &Type) -> bool {
        // All types that can be compared
        matches!(ty, 
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | 
            Type::F32 | Type::F64 | Type::Bool | Type::Str | Type::BigInt | Type::Int
        )
    }
    
    /// Solve trait constraints after type unification
    pub fn solve_trait_constraints(
        &self,
        trait_constraints: &[TraitConstraint],
        subst: &Substitution,
    ) -> Result<(), Vec<TypeError>> {
        let mut errors = Vec::new();
        
        for constraint in trait_constraints {
            // Apply substitution to get the concrete type
            let concrete_ty = constraint.ty.substitute(subst);
            
            // Check if the concrete type satisfies the trait
            if !self.check_trait_constraint(&concrete_ty, &constraint.trait_name) {
                errors.push(TypeError::new(
                    format!(
                        "Type {} does not satisfy trait bound {}: {}",
                        concrete_ty, constraint.trait_name, constraint.ty
                    ),
                    constraint.span,
                ));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    /// Perform Hindley-Milner style type inference on an expression
    /// Returns the inferred type and any generated constraints
    pub fn infer_expr_hm(&mut self, expr: &Expr) -> (Type, Vec<Constraint>) {
        match expr {
            Expr::Int(_, _span) => (Type::I64, vec![]),
            Expr::Float(_, _span) => (Type::F64, vec![]),
            Expr::Bool(_, _span) => (Type::Bool, vec![]),
            Expr::Str(_, _span) => (Type::Str, vec![]),
            Expr::None(_span) => (Type::None, vec![]),
            Expr::BigInt(_, _span) => (Type::BigInt, vec![]),

            Expr::Ident(name, _span) => {
                // Look up the identifier's type
                if let Some(symbol) = self.symbol_table.lookup(name) {
                    if let Some(ty) = symbol.get_type() {
                        // If the type contains type variables, freshen them
                        (self.freshen_type(&ty), vec![])
                    } else {
                        // Unknown type, create a fresh type variable
                        let tvar = self.fresh_type_var();
                        (Type::Var(tvar.clone()), vec![])
                    }
                } else {
                    // Not found, create a fresh type variable
                    let tvar = self.fresh_type_var();
                    (Type::Var(tvar.clone()), vec![])
                }
            }
            
            Expr::List { elements, span: _ } => {
                if elements.is_empty() {
                    // Empty list: [T] where T is fresh
                    let tvar = self.fresh_type_var();
                    (Type::List(Box::new(Type::Var(tvar))), vec![])
                } else {
                    // Infer type from first element
                    let (first_ty, mut constraints) = self.infer_expr_hm(&elements[0]);
                    
                    // All other elements must have the same type
                    for elem in &elements[1..] {
                        let (elem_ty, elem_constraints) = self.infer_expr_hm(elem);
                        constraints.extend(elem_constraints);
                        constraints.push(Constraint::new(first_ty.clone(), elem_ty, elem.span()));
                    }
                    
                    (Type::List(Box::new(first_ty)), constraints)
                }
            }
            
            Expr::Tuple { elements, span: _ } => {
                let mut tuple_types = Vec::new();
                let mut all_constraints = Vec::new();
                
                for elem in elements {
                    let (elem_ty, elem_constraints) = self.infer_expr_hm(elem);
                    tuple_types.push(elem_ty);
                    all_constraints.extend(elem_constraints);
                }
                
                (Type::Tuple(tuple_types), all_constraints)
            }
            
            Expr::Call { func, args, span } => {
                // Check for builtin BigInt functions
                if let Expr::Ident(name, _) = func.as_ref() {
                    if let Some(builtin_sig) = self.get_bigint_builtin_signature(name, args, *span) {
                        let (arg_tys, return_ty) = builtin_sig;
                        let mut constraints = Vec::new();
                        
                        // Constrain arguments to expected types
                        for (arg, expected_arg_ty) in args.iter().zip(arg_tys.iter()) {
                            let (arg_ty, arg_constraints) = self.infer_expr_hm(arg);
                            constraints.extend(arg_constraints);
                            constraints.push(Constraint::new(arg_ty, expected_arg_ty.clone(), arg.span()));
                        }
                        
                        return (return_ty, constraints);
                    }
                }
                
                // Infer function type (non-builtin case)
                let (func_ty, mut constraints) = self.infer_expr_hm(func);

                // Create fresh type variables for argument and return types
                let arg_tys: Vec<Type> = args.iter()
                    .map(|_| Type::Var(self.fresh_type_var()))
                    .collect();
                let return_ty = Type::Var(self.fresh_type_var());

                // Function type should be: arg1 -> arg2 -> ... -> return
                let expected_func_ty = Type::Fn(arg_tys.clone(), Box::new(return_ty.clone()));

                // Constrain the function type
                constraints.push(Constraint::new(func_ty, expected_func_ty, *span));

                // Infer argument types and constrain
                for (arg, expected_arg_ty) in args.iter().zip(arg_tys.iter()) {
                    let (arg_ty, arg_constraints) = self.infer_expr_hm(arg);
                    constraints.extend(arg_constraints);
                    constraints.push(Constraint::new(arg_ty, expected_arg_ty.clone(), arg.span()));
                }

                (return_ty, constraints)
            }
            
            Expr::BinOp { left, op, right, span } => {
                let (left_ty, left_constraints) = self.infer_expr_hm(left);
                let (right_ty, right_constraints) = self.infer_expr_hm(right);
                let mut constraints = left_constraints;
                constraints.extend(right_constraints);
                
                // Determine result type based on operator
                let result_ty = match op {
                    // Arithmetic operators: require numeric types, result is the common type
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::FloorDiv | BinOp::Pow => {
                        // Constrain both operands to be the same type
                        constraints.push(Constraint::new(left_ty.clone(), right_ty.clone(), *span));
                        // Result type is the same as operands
                        left_ty
                    }
                    // Comparison operators: return bool
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                        // For equality, types should be compatible
                        constraints.push(Constraint::new(left_ty.clone(), right_ty.clone(), *span));
                        Type::Bool
                    }
                    // Logical operators: require bool operands, return bool
                    BinOp::And | BinOp::Or => {
                        constraints.push(Constraint::new(left_ty.clone(), Type::Bool, *span));
                        constraints.push(Constraint::new(right_ty.clone(), Type::Bool, *span));
                        Type::Bool
                    }
                    // Default: i64
                    _ => Type::I64,
                };
                
                (result_ty, constraints)
            }
            
            Expr::UnaryOp { op, operand, span } => {
                let (operand_ty, mut constraints) = self.infer_expr_hm(operand);
                
                let result_ty = match op {
                    UnaryOp::Neg | UnaryOp::Pos => {
                        // Numeric operand required
                        constraints.push(Constraint::new(operand_ty.clone(), Type::I64, *span));
                        operand_ty
                    }
                    UnaryOp::Not => {
                        // Bool operand required
                        constraints.push(Constraint::new(operand_ty.clone(), Type::Bool, *span));
                        Type::Bool
                    }
                    UnaryOp::Unwrap => {
                        // Operand must be Result[T, E], result is T
                        let ok_tvar = Type::Var(self.fresh_type_var());
                        let err_tvar = Type::Var(self.fresh_type_var());
                        let result_ty = Type::Result(Box::new(ok_tvar.clone()), Box::new(err_tvar.clone()));
                        constraints.push(Constraint::new(operand_ty.clone(), result_ty, *span));
                        ok_tvar
                    }
                    UnaryOp::UnwrapOrDefault => {
                        // Operand must be Result[T, E], result is T
                        let ok_tvar = Type::Var(self.fresh_type_var());
                        let err_tvar = Type::Var(self.fresh_type_var());
                        let result_ty = Type::Result(Box::new(ok_tvar.clone()), Box::new(err_tvar.clone()));
                        constraints.push(Constraint::new(operand_ty.clone(), result_ty, *span));
                        ok_tvar
                    }
                    _ => Type::I64,
                };
                
                (result_ty, constraints)
            }
            
            Expr::Index { obj, index, span } => {
                let (obj_ty, mut constraints) = self.infer_expr_hm(obj);
                let (index_ty, index_constraints) = self.infer_expr_hm(index);
                constraints.extend(index_constraints);
                
                // Index should be i64
                constraints.push(Constraint::new(index_ty, Type::I64, *span));
                
                // Object should be List[T], result is T
                let elem_tvar = Type::Var(self.fresh_type_var());
                constraints.push(Constraint::new(obj_ty, Type::List(Box::new(elem_tvar.clone())), *span));
                
                (elem_tvar, constraints)
            }
            
            Expr::Attribute { obj, attr: _, span: _ } => {
                let (_obj_ty, constraints) = self.infer_expr_hm(obj);

                // Attribute access result type depends on the object type
                // For now, use a fresh type variable
                let result_ty = Type::Var(self.fresh_type_var());

                // TODO: Add proper field/method lookup here
                // For class instances, look up the field/method type
                
                (result_ty, constraints)
            }
            
            Expr::Conditional { condition, then_expr, else_expr, span } => {
                let (cond_ty, mut cond_constraints) = self.infer_expr_hm(condition);
                let (then_ty, then_constraints) = self.infer_expr_hm(then_expr);
                let (else_ty, else_constraints) = self.infer_expr_hm(else_expr);
                
                cond_constraints.extend(then_constraints);
                cond_constraints.extend(else_constraints);
                
                // Condition must be bool
                cond_constraints.push(Constraint::new(cond_ty, Type::Bool, *span));
                
                // Then and else branches must have the same type
                cond_constraints.push(Constraint::new(then_ty.clone(), else_ty, *span));
                
                (then_ty, cond_constraints)
            }
            
            Expr::Lambda { params, body, span } => {
                // Create type variables for each parameter
                let param_tys: Vec<Type> = params.iter()
                    .map(|_| Type::Var(self.fresh_type_var()))
                    .collect();
                let return_ty = Type::Var(self.fresh_type_var());
                
                // Add parameters to symbol table temporarily
                self.symbol_table.enter_scope();
                for (i, param) in params.iter().enumerate() {
                    let symbol = Symbol::new(
                        param.clone(),
                        SymbolKind::Variable { mutable: true, type_ann: Some(param_tys[i].clone()) },
                        *span,
                        self.symbol_table.current_scope_id(),
                    );
                    let _ = self.symbol_table.insert(symbol);
                }
                
                // Infer body type
                let (body_ty, mut constraints) = self.infer_expr_hm(body);
                
                // Constrain body type to return type
                constraints.push(Constraint::new(body_ty, return_ty.clone(), *span));
                
                self.symbol_table.exit_scope();
                
                (Type::Fn(param_tys, Box::new(return_ty)), constraints)
            }
            
            // For other expressions, use simple inference
            _ => {
                // Fall back to the existing inference
                (self.infer_expr_type(expr).unwrap_or(Type::Infer), vec![])
            }
        }
    }
    
    /// Generate a fresh type variable name
    fn fresh_type_var(&self) -> String {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("t{}", id)
    }
    
    /// Freshen a type by replacing all type variables with fresh ones
    fn freshen_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(_name) => Type::Var(self.fresh_type_var()),
            Type::TypeParam { name, bounds } => Type::TypeParam {
                name: name.clone(),
                bounds: bounds.iter().map(|b| self.freshen_type(b)).collect(),
            },
            Type::List(inner) => Type::List(Box::new(self.freshen_type(inner))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.freshen_type(k)),
                Box::new(self.freshen_type(v)),
            ),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| self.freshen_type(t)).collect()),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| self.freshen_type(p)).collect(),
                Box::new(self.freshen_type(ret)),
            ),
            Type::GenericApp { name, type_args } => Type::GenericApp {
                name: name.clone(),
                type_args: type_args.iter().map(|t| self.freshen_type(t)).collect(),
            },
            Type::Result(ok, err) => Type::Result(
                Box::new(self.freshen_type(ok)),
                Box::new(self.freshen_type(err)),
            ),
            Type::Union(variants) => Type::Union(variants.iter().map(|t| self.freshen_type(t)).collect()),
            _ => ty.clone(),
        }
    }
    
    /// Unify a list of constraints to produce a substitution
    pub fn unify(&self, constraints: Vec<Constraint>) -> Result<Substitution, TypeError> {
        let mut subst = Substitution::new();
        
        for constraint in constraints {
            self.unify_types(&mut subst, constraint.ty1, constraint.ty2, constraint.span)?;
        }
        
        Ok(subst)
    }
    
    /// Unify two types, extending the substitution
    fn unify_types(
        &self,
        subst: &mut Substitution,
        ty1: Type,
        ty2: Type,
        span: crate::utils::Span,
    ) -> Result<(), TypeError> {
        // Apply current substitution
        let ty1 = self.apply_subst(&subst, ty1);
        let ty2 = self.apply_subst(&subst, ty2);
        
        if ty1 == ty2 {
            return Ok(());
        }
        
        match (ty1, ty2) {
            // Type variable cases
            (Type::Var(var), ty) | (ty, Type::Var(var)) => {
                // Occurs check: prevent infinite types
                if self.occurs_in(&var, &ty) {
                    return Err(TypeError::new(
                        format!("Occurs check failed: {} occurs in {}", var, ty),
                        span,
                    ));
                }
                subst.insert(var, ty);
                Ok(())
            }
            
            // Compound type cases
            (Type::List(inner1), Type::List(inner2)) => {
                self.unify_types(subst, *inner1, *inner2, span)
            }
            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
                self.unify_types(subst, *k1, *k2, span)?;
                self.unify_types(subst, *v1, *v2, span)
            }
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(TypeError::new(
                        format!("Tuple arity mismatch: {} vs {}", types1.len(), types2.len()),
                        span,
                    ));
                }
                for (t1, t2) in types1.into_iter().zip(types2.into_iter()) {
                    self.unify_types(subst, t1, t2, span)?;
                }
                Ok(())
            }
            (Type::Fn(params1, ret1), Type::Fn(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(TypeError::new(
                        format!("Function arity mismatch: {} vs {}", params1.len(), params2.len()),
                        span,
                    ));
                }
                for (p1, p2) in params1.into_iter().zip(params2.into_iter()) {
                    self.unify_types(subst, p1, p2, span)?;
                }
                self.unify_types(subst, *ret1, *ret2, span)
            }
            (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                self.unify_types(subst, *ok1, *ok2, span)?;
                self.unify_types(subst, *err1, *err2, span)
            }
            (Type::Union(vars1), Type::Union(vars2)) => {
                if vars1.len() != vars2.len() {
                    return Err(TypeError::new(
                        format!("Union arity mismatch: {} vs {}", vars1.len(), vars2.len()),
                        span,
                    ));
                }
                for (v1, v2) in vars1.into_iter().zip(vars2.into_iter()) {
                    self.unify_types(subst, v1, v2, span)?;
                }
                Ok(())
            }
            
            // Primitive types must match exactly
            (Type::I8, Type::I8) | (Type::I16, Type::I16) | (Type::I32, Type::I32) |
            (Type::I64, Type::I64) | (Type::F32, Type::F32) | (Type::F64, Type::F64) |
            (Type::Bool, Type::Bool) | (Type::Str, Type::Str) | (Type::BigInt, Type::BigInt) |
            (Type::None, Type::None) | (Type::Int, Type::Int) => Ok(()),
            
            // Infer is compatible with anything
            (Type::Infer, _) | (_, Type::Infer) => Ok(()),
            
            // Error case: types don't unify
            (ty1, ty2) => Err(TypeError::new(
                format!("Type mismatch: cannot unify {} with {}", ty1, ty2),
                span,
            )),
        }
    }
    
    /// Check if a type variable occurs in a type (occurs check)
    fn occurs_in(&self, var: &str, ty: &Type) -> bool {
        match ty {
            Type::Var(name) => name == var,
            Type::TypeParam { name, .. } => name == var,
            Type::List(inner) => self.occurs_in(var, inner),
            Type::Dict(k, v) => self.occurs_in(var, k) || self.occurs_in(var, v),
            Type::Tuple(types) => types.iter().any(|t| self.occurs_in(var, t)),
            Type::Fn(params, ret) => {
                params.iter().any(|p| self.occurs_in(var, p)) || self.occurs_in(var, ret)
            }
            Type::Result(ok, err) => self.occurs_in(var, ok) || self.occurs_in(var, err),
            Type::Union(variants) => variants.iter().any(|t| self.occurs_in(var, t)),
            Type::GenericApp { type_args, .. } => type_args.iter().any(|t| self.occurs_in(var, t)),
            Type::Array(elem, _) => self.occurs_in(var, elem),
            Type::Optional(inner) => self.occurs_in(var, inner),
            Type::Future(inner) => self.occurs_in(var, inner),
            Type::Struct { fields, .. } => fields.iter().any(|(_, t)| self.occurs_in(var, t)),
            _ => false,
        }
    }
    
    /// Apply a substitution to a type
    fn apply_subst(&self, subst: &Substitution, ty: Type) -> Type {
        ty.substitute(subst)
    }

    /// Get the signature of a BigInt builtin function
    /// Returns (argument_types, return_type) if it's a known builtin
    fn get_bigint_builtin_signature(
        &self,
        name: &str,
        args: &[Expr],
        _span: crate::utils::Span,
    ) -> Option<(Vec<Type>, Type)> {
        match name {
            // BigInt constructor: BigInt(str) -> BigInt
            "BigInt" => {
                if args.len() == 1 {
                    Some((vec![Type::Str], Type::BigInt))
                } else {
                    None
                }
            }
            // str_bigint: str_bigint(BigInt) -> Str
            "str_bigint" => {
                if args.len() == 1 {
                    Some((vec![Type::BigInt], Type::Str))
                } else {
                    None
                }
            }
            // int_bigint: int_bigint(BigInt) -> I64
            "int_bigint" => {
                if args.len() == 1 {
                    Some((vec![Type::BigInt], Type::I64))
                } else {
                    None
                }
            }
            // abs_bigint: abs_bigint(BigInt) -> BigInt
            "abs_bigint" => {
                if args.len() == 1 {
                    Some((vec![Type::BigInt], Type::BigInt))
                } else {
                    None
                }
            }
            // pow_bigint: pow_bigint(BigInt, BigInt) -> BigInt
            "pow_bigint" => {
                if args.len() == 2 {
                    Some((vec![Type::BigInt, Type::BigInt], Type::BigInt))
                } else {
                    None
                }
            }
            // sqrt_bigint: sqrt_bigint(BigInt) -> BigInt
            "sqrt_bigint" => {
                if args.len() == 1 {
                    Some((vec![Type::BigInt], Type::BigInt))
                } else {
                    None
                }
            }
            // min_bigint: min_bigint(BigInt, BigInt) -> BigInt
            "min_bigint" => {
                if args.len() == 2 {
                    Some((vec![Type::BigInt, Type::BigInt], Type::BigInt))
                } else {
                    None
                }
            }
            // max_bigint: max_bigint(BigInt, BigInt) -> BigInt
            "max_bigint" => {
                if args.len() == 2 {
                    Some((vec![Type::BigInt, Type::BigInt], Type::BigInt))
                } else {
                    None
                }
            }
            // Not a recognized BigInt builtin
            _ => None,
        }
    }

    /// Perform complete type inference on an expression
    /// Returns the fully inferred type (with all type variables substituted)
    pub fn infer_expr_complete(&mut self, expr: &Expr) -> Result<Type, Vec<TypeError>> {
        let (ty, constraints) = self.infer_expr_hm(expr);

        // Unify constraints
        let subst = self.unify(constraints)
            .map_err(|e| vec![e])?;

        // Apply substitution to get final type
        let final_ty = self.apply_subst(&subst, ty);

        Ok(final_ty)
    }
}
