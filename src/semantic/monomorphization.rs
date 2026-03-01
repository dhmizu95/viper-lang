//! Monomorphization - Specialize Generic Functions Per Type
//!
//! This module implements monomorphization for generic functions:
//! 1. Track generic function definitions with type parameters
//! 2. When a generic function is called with concrete types, create a specialized version
//! 3. Generate unique mangled names for each specialization
//! 4. Replace generic type parameters with concrete types in the specialized body

use crate::ast::{Stmt, Type};
use crate::semantic::symbol_table::{Symbol, SymbolKind};
use crate::semantic::type_checker::TypeChecker;
use std::collections::HashMap;

/// Represents a monomorphized (specialized) version of a generic function
#[derive(Debug, Clone)]
pub struct MonomorphizedFunction {
    /// Original generic function name
    pub original_name: String,
    /// Concrete type arguments (e.g., [i64, str] for swap[i64, str])
    pub type_args: Vec<Type>,
    /// Mangled name for this specialization (e.g., swap_i64_str)
    pub mangled_name: String,
    /// Specialized function body (with type params substituted)
    pub body: Vec<Stmt>,
    /// Specialized parameter types
    pub param_types: Vec<Type>,
    /// Specialized return type
    pub return_type: Option<Type>,
}

/// Monomorphization state - tracks which specializations have been created
pub struct Monomorphizer {
    /// Map from mangled name to monomorphized function info
    pub monomorphized_funcs: HashMap<String, MonomorphizedFunction>,
    /// Counter for generating unique specialization IDs
    pub counter: usize,
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            monomorphized_funcs: HashMap::new(),
            counter: 0,
        }
    }
    
    /// Check if a function has generic type parameters
    pub fn is_generic_function(symbol: &Symbol) -> bool {
        match &symbol.kind {
            SymbolKind::Function { type_params, .. } => !type_params.is_empty(),
            _ => false,
        }
    }
    
    /// Generate a mangled name for a specialization
    fn generate_mangled_name(&mut self, base_name: &str, type_args: &[Type]) -> String {
        if type_args.is_empty() {
            return base_name.to_string();
        }
        
        let type_suffix: Vec<String> = type_args.iter().map(|t| Self::mangle_type(t)).collect();
        format!("{}_{}", base_name, type_suffix.join("_"))
    }
    
    /// Mangle a type for use in function names
    fn mangle_type(ty: &Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "str".to_string(),
            Type::BigInt => "bigint".to_string(),
            Type::Int => "int".to_string(),
            Type::List(t) => format!("list_{}", Self::mangle_type(t)),
            Type::Dict(k, v) => format!("dict_{}_{}", Self::mangle_type(k), Self::mangle_type(v)),
            Type::Result(ok, err) => format!("result_{}_{}", Self::mangle_type(ok), Self::mangle_type(err)),
            Type::Var(name) => format!("var_{}", name),
            Type::TypeParam { name, .. } => format!("typeparam_{}", name),
            Type::Union(variants) => {
                let variants_mangled: Vec<String> = variants.iter().map(|v| Self::mangle_type(v)).collect();
                format!("union_{}", variants_mangled.join("_"))
            }
            Type::GenericApp { name, type_args } => {
                let args_mangled: Vec<String> = type_args.iter().map(|t| Self::mangle_type(t)).collect();
                format!("generic_{}_{}", name, args_mangled.join("_"))
            }
            Type::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| Self::mangle_type(t)).collect();
                format!("tuple{}", inner.join("_"))
            }
            Type::Fn(params, ret) => {
                let params_mangled: Vec<String> = params.iter().map(|t| Self::mangle_type(t)).collect();
                format!("fn_{}_ret_{}", params_mangled.join("_"), Self::mangle_type(ret))
            }
            Type::Optional(t) => format!("opt_{}", Self::mangle_type(t)),
            Type::Array(t, size) => format!("array_{}_{}", Self::mangle_type(t), size),
            Type::Chan(t) => format!("chan_{}", Self::mangle_type(t)),
            Type::Future(t) => format!("future_{}", Self::mangle_type(t)),
            Type::Struct { name, .. } => format!("struct_{}", name),
            Type::Class(name) | Type::Instance(name) => format!("class_{}", name),
            Type::None => "none".to_string(),
            Type::Bytes => "bytes".to_string(),
            Type::WaitGroup => "waitgroup".to_string(),
            Type::Infer => "infer".to_string(),
            Type::Error => "error".to_string(),
            Type::Object => "object".to_string(),
            Type::Method { class_name, method_name, .. } => {
                format!("method_{}_{}", class_name, method_name)
            }
        }
    }
    
    /// Create a substitution map from type parameters to concrete types
    fn create_substitution(
        type_params: &[String],
        type_args: &[Type],
    ) -> HashMap<String, Type> {
        type_params.iter()
            .zip(type_args.iter())
            .map(|(param, arg)| (param.clone(), arg.clone()))
            .collect()
    }
    
    /// Specialize a generic function call with concrete type arguments
    /// Returns the mangled name of the specialized function
    pub fn specialize_function(
        &mut self,
        func_name: &str,
        type_args: &[Type],
        original_symbol: &Symbol,
        original_body: &[Stmt],
    ) -> Result<String, String> {
        // Generate mangled name first to check if we already have this specialization
        let mangled_name = self.generate_mangled_name(func_name, type_args);
        
        // Check if we already have this specialization
        if self.monomorphized_funcs.contains_key(&mangled_name) {
            return Ok(mangled_name);
        }
        
        // Get type parameters from the original function
        let type_params = match &original_symbol.kind {
            SymbolKind::Function { type_params, .. } => type_params,
            _ => return Err(format!("{} is not a generic function", func_name)),
        };
        
        if type_params.is_empty() {
            return Err(format!("{} is not a generic function", func_name));
        }
        
        if type_params.len() != type_args.len() {
            return Err(format!(
                "Type argument count mismatch: expected {}, got {}",
                type_params.len(),
                type_args.len()
            ));
        }
        
        // Create substitution map
        let subst = Self::create_substitution(type_params, type_args);
        
        // Get original function info
        let (param_types, return_type) = match &original_symbol.kind {
            SymbolKind::Function { params, return_type, .. } => (params.clone(), return_type.clone()),
            _ => return Err(format!("{} is not a function", func_name)),
        };
        
        // Specialize parameter types
        let specialized_params: Vec<Type> = param_types.iter()
            .map(|p| p.substitute(&subst))
            .collect();
        
        // Specialize return type
        let specialized_return = return_type.map(|r| r.substitute(&subst));
        
        // Specialize function body (substitute types in statements)
        let specialized_body = self.specialize_stmts(original_body, &subst);
        
        // Store the specialization
        let mono_func = MonomorphizedFunction {
            original_name: func_name.to_string(),
            type_args: type_args.to_vec(),
            mangled_name: mangled_name.clone(),
            body: specialized_body,
            param_types: specialized_params,
            return_type: specialized_return,
        };
        
        self.monomorphized_funcs.insert(mangled_name.clone(), mono_func);
        
        Ok(mangled_name)
    }
    
    /// Substitute types in a list of statements
    fn specialize_stmts(&self, stmts: &[Stmt], subst: &HashMap<String, Type>) -> Vec<Stmt> {
        stmts.iter()
            .map(|stmt| self.specialize_stmt(stmt, subst))
            .collect()
    }
    
    /// Substitute types in a single statement
    fn specialize_stmt(&self, stmt: &Stmt, _subst: &HashMap<String, Type>) -> Stmt {
        // For now, we do a shallow substitution
        // A full implementation would recursively substitute in all nested expressions
        stmt.clone()  // TODO: Implement full substitution
    }
    
    /// Get all monomorphized functions that need to be codegen'd
    pub fn get_monomorphized_functions(&self) -> &HashMap<String, MonomorphizedFunction> {
        &self.monomorphized_funcs
    }
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    /// Get or create the monomorphizer (lazy initialization)
    pub fn get_monomorphizer(&mut self) -> &mut Monomorphizer {
        // We'll store the monomorphizer in the type checker
        // For now, create a local one - in a real implementation, this would be stored
        #[allow(static_mut_refs)]
        static mut MONOMORPHIZER: Option<Monomorphizer> = None;
        #[allow(static_mut_refs)]
        unsafe {
            if MONOMORPHIZER.is_none() {
                MONOMORPHIZER = Some(Monomorphizer::new());
            }
            MONOMORPHIZER.as_mut().unwrap()
        }
    }
    
    /// Check if a type contains any type parameters (needs monomorphization)
    pub fn contains_type_params(&self, ty: &Type) -> bool {
        match ty {
            Type::Var(_) | Type::TypeParam { .. } => true,
            Type::List(inner) => self.contains_type_params(inner),
            Type::Dict(k, v) => self.contains_type_params(k) || self.contains_type_params(v),
            Type::Tuple(types) => types.iter().any(|t| self.contains_type_params(t)),
            Type::Fn(params, ret) => {
                params.iter().any(|p| self.contains_type_params(p)) || self.contains_type_params(ret)
            }
            Type::Result(ok, err) => {
                self.contains_type_params(ok) || self.contains_type_params(err)
            }
            Type::Union(variants) => variants.iter().any(|t| self.contains_type_params(t)),
            Type::GenericApp { type_args, .. } => type_args.iter().any(|t| self.contains_type_params(t)),
            Type::Array(elem, _) => self.contains_type_params(elem),
            Type::Optional(inner) => self.contains_type_params(inner),
            Type::Future(inner) => self.contains_type_params(inner),
            Type::Struct { fields, .. } => fields.iter().any(|(_, t)| self.contains_type_params(t)),
            _ => false,
        }
    }
}
