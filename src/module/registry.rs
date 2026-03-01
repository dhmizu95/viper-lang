//! Module Registry for Viper
//!
//! Tracks imported modules and their exported symbols during code generation

use std::collections::HashMap;
use crate::ast::{Stmt, Type};

/// Information about an exported symbol from a module
#[derive(Debug, Clone)]
pub struct ExportedSymbol {
    pub name: String,
    pub symbol_type: Option<Type>,
    pub is_function: bool,
    pub is_class: bool,
    pub is_constant: bool,
}

/// Information about an imported module
#[derive(Debug, Clone)]
pub struct ImportedModule {
    pub name: String,
    pub alias: Option<String>,
    pub symbols: HashMap<String, ExportedSymbol>,
    pub initialized: bool,
}

impl ImportedModule {
    pub fn new(name: String, alias: Option<String>) -> Self {
        Self {
            name,
            alias,
            symbols: HashMap::new(),
            initialized: false,
        }
    }
    
    pub fn qualified_name(&self) -> String {
        self.alias.clone().unwrap_or_else(|| self.name.clone())
    }
}

/// Module registry for tracking imports during code generation
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, ImportedModule>,
    search_paths: Vec<String>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            search_paths: Vec::new(),
        }
    }
    
    /// Add a search path for modules
    pub fn add_search_path(&mut self, path: String) {
        self.search_paths.push(path);
    }
    
    /// Register an imported module
    pub fn register_module(&mut self, name: String, alias: Option<String>) {
        let module = ImportedModule::new(name.clone(), alias);
        self.modules.insert(name, module);
    }
    
    /// Add an exported symbol to a module
    pub fn add_export(&mut self, module_name: &str, symbol: ExportedSymbol) {
        if let Some(module) = self.modules.get_mut(module_name) {
            module.symbols.insert(symbol.name.clone(), symbol);
        }
    }
    
    /// Check if a module is imported
    pub fn is_imported(&self, name: &str) -> bool {
        self.modules.values().any(|m| m.name == name || m.alias.as_deref() == Some(name))
    }
    
    /// Get a module by name or alias
    pub fn get_module(&self, name: &str) -> Option<&ImportedModule> {
        self.modules.values().find(|m| m.name == name || m.alias.as_deref() == Some(name))
    }
    
    /// Get mutable reference to a module
    pub fn get_module_mut(&mut self, name: &str) -> Option<&mut ImportedModule> {
        self.modules.values_mut().find(|m| m.name == name || m.alias.as_deref() == Some(name))
    }
    
    /// Check if a symbol is exported from a module
    pub fn has_export(&self, module_name: &str, symbol_name: &str) -> bool {
        if let Some(module) = self.get_module(module_name) {
            return module.symbols.contains_key(symbol_name);
        }
        false
    }
    
    /// Get an exported symbol
    pub fn get_export(&self, module_name: &str, symbol_name: &str) -> Option<&ExportedSymbol> {
        if let Some(module) = self.get_module(module_name) {
            return module.symbols.get(symbol_name);
        }
        None
    }
    
    /// Mark a module as initialized
    pub fn mark_initialized(&mut self, module_name: &str) {
        if let Some(module) = self.get_module_mut(module_name) {
            module.initialized = true;
        }
    }
    
    /// Get all imported modules
    pub fn modules(&self) -> &HashMap<String, ImportedModule> {
        &self.modules
    }
    
    /// Analyze a module's AST to extract exports
    pub fn analyze_exports(&mut self, module_name: &str, statements: &[Stmt]) {
        for stmt in statements {
            match stmt {
                Stmt::Function { name, return_type, .. } => {
                    self.add_export(module_name, ExportedSymbol {
                        name: name.clone(),
                        symbol_type: return_type.clone(),
                        is_function: true,
                        is_class: false,
                        is_constant: false,
                    });
                }
                Stmt::Class { name, .. } => {
                    self.add_export(module_name, ExportedSymbol {
                        name: name.clone(),
                        symbol_type: Some(Type::Class(name.clone())),
                        is_function: false,
                        is_class: true,
                        is_constant: false,
                    });
                }
                Stmt::Declare { name, type_ann, .. } => {
                    self.add_export(module_name, ExportedSymbol {
                        name: name.clone(),
                        symbol_type: type_ann.clone(),
                        is_function: false,
                        is_class: false,
                        is_constant: true,
                    });
                }
                _ => {}
            }
        }
    }
}
