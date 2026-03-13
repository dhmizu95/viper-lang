use crate::ast::{Module, Type};
use crate::module::{ModuleLoader, ModuleRegistry, ModuleSearchPath};
use crate::semantic::symbol_table::{Symbol, SymbolTable};
use std::collections::HashMap;
use std::path::Path;

pub mod compatibility;
pub mod exprs;
pub mod hindley_milner;
pub mod infer;
pub mod overload;
pub mod stmts;

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

/// Type checker for Viper programs
pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    pub errors: Vec<TypeError>,
    /// Map from expression to inferred type
    pub expr_types: HashMap<usize, Type>,
    /// Map from channel variable name to element type (for Chan[T] inference)
    #[allow(dead_code)]
    pub channel_types: HashMap<String, Type>,
    /// Current function's return type (for context-sensitive inference)
    pub current_return_type: Option<Type>,
    /// Module loader for handling imports
    pub module_loader: ModuleLoader,
    /// Module registry for tracking imports
    pub module_registry: ModuleRegistry,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            expr_types: HashMap::new(),
            channel_types: HashMap::new(),
            current_return_type: None,
            module_loader: ModuleLoader::new(),
            module_registry: ModuleRegistry::new(),
        }
    }

    /// Create a new type checker with a specific input path for module resolution
    pub fn with_input_path(input_path: &Path) -> Self {
        let mut search_path = ModuleSearchPath::new();
        if let Some(parent) = input_path.parent() {
            search_path.add_path(parent.to_path_buf());
        }

        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            expr_types: HashMap::new(),
            channel_types: HashMap::new(),
            current_return_type: None,
            module_loader: ModuleLoader::with_search_path(search_path),
            module_registry: ModuleRegistry::new(),
        }
    }

    /// Type check a complete module
    pub fn check(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        self.errors.clear();

        // First pass: process imports
        self.process_imports(module)?;

        // Second pass: collect function declarations
        for stmt in &module.statements {
            if let crate::ast::Stmt::Function {
                name, params, return_type, span, type_params, ..
            } = stmt
            {
                // Normalize parameter types
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        p.type_ann.as_ref().map(|t| self.normalize_type(t)).unwrap_or(Type::Infer)
                    })
                    .collect();

                // Normalize return type (convert GenericApp Result to Type::Result)
                let normalized_return_type = return_type.as_ref().map(|t| self.normalize_type(t));

                // type_params is already Vec<String>
                let symbol = Symbol::new_function(
                    name.clone(),
                    param_types,
                    normalized_return_type,
                    *span,
                    self.symbol_table.current_scope_id(),
                    type_params.clone(),
                );
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e.to_string(), *span));
                }
            } else if let crate::ast::Stmt::Extern { name, params, return_type, span, .. } = stmt {
                // Normalize parameter types
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        p.type_ann.as_ref().map(|t| self.normalize_type(t)).unwrap_or(Type::Infer)
                    })
                    .collect();

                // Normalize return type
                let normalized_return_type = return_type.as_ref().map(|t| self.normalize_type(t));

                let symbol = Symbol::new_function(
                    name.clone(),
                    param_types,
                    normalized_return_type,
                    *span,
                    self.symbol_table.current_scope_id(),
                    vec![],
                );
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e.to_string(), *span));
                }
            }
        }

        // Third pass: type check all statements
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

    /// Normalize a type - convert GenericApp Result[T, E] to Type::Result(T, E)
    fn normalize_type(&self, ty: &Type) -> Type {
        match ty {
            Type::GenericApp { name, type_args } => {
                if name == "Result" && type_args.len() == 2 {
                    Type::Result(
                        Box::new(self.normalize_type(&type_args[0])),
                        Box::new(self.normalize_type(&type_args[1])),
                    )
                } else if name == "List" && type_args.len() == 1 {
                    Type::List(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Dict" && type_args.len() == 2 {
                    Type::Dict(
                        Box::new(self.normalize_type(&type_args[0])),
                        Box::new(self.normalize_type(&type_args[1])),
                    )
                } else if name == "Optional" && type_args.len() == 1 {
                    Type::Optional(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Future" && type_args.len() == 1 {
                    Type::Future(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Chan" && type_args.len() == 1 {
                    Type::Chan(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Array" && type_args.len() == 2 {
                    // Array[T, N] - but N should be a constant, not a type
                    // For now, just normalize the element type
                    Type::Array(
                        Box::new(self.normalize_type(&type_args[0])),
                        0, // Size would need special handling
                    )
                } else {
                    ty.clone()
                }
            }
            // Recursively normalize nested types
            Type::List(inner) => Type::List(Box::new(self.normalize_type(inner))),
            Type::Dict(k, v) => {
                Type::Dict(Box::new(self.normalize_type(k)), Box::new(self.normalize_type(v)))
            }
            Type::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.normalize_type(t)).collect())
            }
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| self.normalize_type(p)).collect(),
                Box::new(self.normalize_type(ret)),
            ),
            Type::Union(variants) => {
                Type::Union(variants.iter().map(|t| self.normalize_type(t)).collect())
            }
            Type::Optional(inner) => Type::Optional(Box::new(self.normalize_type(inner))),
            Type::Future(inner) => Type::Future(Box::new(self.normalize_type(inner))),
            Type::Chan(inner) => Type::Chan(Box::new(self.normalize_type(inner))),
            Type::Array(elem, size) => Type::Array(Box::new(self.normalize_type(elem)), *size),
            _ => ty.clone(),
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

    /// Process import statements and load modules
    fn process_imports(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        for stmt in &module.statements {
            match stmt {
                crate::ast::Stmt::Import { module: mod_name, alias, span } => {
                    // Load the module
                    match self.module_loader.load_module(mod_name) {
                        Ok(loaded_module) => {
                            // Register the module
                            self.module_registry.register_module(mod_name.clone(), alias.clone());

                            // Analyze exports from the loaded module
                            self.module_registry.analyze_exports(
                                alias.as_deref().unwrap_or(mod_name),
                                &loaded_module.ast.statements,
                            );

                            // Add module as a symbol in the current scope
                            let module_symbol = crate::semantic::symbol_table::Symbol::new(
                                alias.as_deref().unwrap_or(mod_name).to_string(),
                                crate::semantic::symbol_table::SymbolKind::Module {
                                    name: mod_name.clone(),
                                },
                                *span,
                                self.symbol_table.current_scope_id(),
                            );
                            if let Err(e) = self.symbol_table.insert(module_symbol) {
                                self.errors.push(TypeError::new(e.to_string(), *span));
                            }
                        }
                        Err(e) => {
                            self.errors.push(TypeError::new(
                                format!("Failed to load module '{}': {}", mod_name, e),
                                *span,
                            ));
                        }
                    }
                }
                crate::ast::Stmt::FromImport { module: mod_name, names, span } => {
                    // Load the module
                    match self.module_loader.load_module(mod_name) {
                        Ok(loaded_module) => {
                            // Register the module
                            self.module_registry.register_module(mod_name.clone(), None);

                            // Analyze exports
                            self.module_registry
                                .analyze_exports(mod_name, &loaded_module.ast.statements);

                            // Add each imported name to the current scope
                            for (name, alias) in names {
                                let import_name = alias.as_deref().unwrap_or(name);

                                // Check if the symbol exists in the module
                                if let Some(export) =
                                    self.module_registry.get_export(mod_name, name)
                                {
                                    // Create a symbol for the imported item
                                    let symbol_kind = if export.is_function {
                                        crate::semantic::symbol_table::SymbolKind::Function {
                                            params: vec![],
                                            return_type: export.symbol_type.clone(),
                                            mangled_name: format!("__import_{}_{}", mod_name, name),
                                            type_params: vec![],
                                        }
                                    } else if export.is_class {
                                        crate::semantic::symbol_table::SymbolKind::Class {
                                            name: name.clone(),
                                            bases: vec![],
                                            fields: vec![],
                                            methods: vec![],
                                            method_mangles: vec![],
                                        }
                                    } else {
                                        crate::semantic::symbol_table::SymbolKind::Variable {
                                            mutable: false,
                                            type_ann: export.symbol_type.clone(),
                                        }
                                    };

                                    let symbol = crate::semantic::symbol_table::Symbol::new(
                                        import_name.to_string(),
                                        symbol_kind,
                                        *span,
                                        self.symbol_table.current_scope_id(),
                                    );
                                    if let Err(e) = self.symbol_table.insert(symbol) {
                                        self.errors.push(TypeError::new(e.to_string(), *span));
                                    }
                                } else {
                                    self.errors.push(TypeError::new(
                                        format!(
                                            "'{}' is not exported from module '{}'",
                                            name, mod_name
                                        ),
                                        *span,
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            self.errors.push(TypeError::new(
                                format!("Failed to load module '{}': {}", mod_name, e),
                                *span,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
