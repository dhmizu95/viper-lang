//! Module Loader for Viper
//!
//! This module handles:
//! - Finding and loading .vp module files
//! - Managing module search paths
//! - Caching loaded modules
//! - Resolving import paths

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::ast::Module;
use crate::parser::Parser;

/// Information about a loaded module
#[derive(Debug)]
pub struct LoadedModule {
    pub name: String,
    pub path: PathBuf,
    pub ast: Module,
    pub initialized: bool,
}

/// Module search path configuration
#[derive(Debug, Clone)]
pub struct ModuleSearchPath {
    paths: Vec<PathBuf>,
}

impl ModuleSearchPath {
    pub fn new() -> Self {
        let mut paths = Vec::new();
        
        // Add current directory
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd);
        }
        
        // Add std directory relative to current directory
        if let Ok(cwd) = std::env::current_dir() {
            let std_path = cwd.join("std");
            if std_path.exists() {
                paths.push(std_path);
            }
        }
        
        // Add std/core for core modules
        if let Ok(cwd) = std::env::current_dir() {
            let std_core_path = cwd.join("std").join("core");
            if std_core_path.exists() {
                paths.push(std_core_path);
            }
        }
        
        // Check for VIPERPATH environment variable
        if let Ok(path_str) = std::env::var("VIPERPATH") {
            for path in path_str.split(':') {
                let p = PathBuf::from(path);
                if p.exists() {
                    paths.push(p);
                }
            }
        }
        
        Self { paths }
    }
    
    pub fn add_path(&mut self, path: PathBuf) {
        if path.exists() && !self.paths.contains(&path) {
            self.paths.push(path);
        }
    }
    
    /// Find a module file by name
    pub fn find_module(&self, module_name: &str) -> Option<PathBuf> {
        // Convert module name to path (e.g., "core.math" -> "core/math.vp")
        let module_path = module_name.replace('.', "/");
        
        // Try different file locations
        let candidates = vec![
            format!("{}.vp", module_path),
            format!("{}/__init__.vp", module_path),
            format!("core/{}.vp", module_path),
        ];
        
        for base_path in &self.paths {
            for candidate in &candidates {
                let full_path = base_path.join(candidate);
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }
        
        None
    }
}

/// Module loader and cache
pub struct ModuleLoader {
    search_path: ModuleSearchPath,
    loaded_modules: HashMap<String, LoadedModule>,
    loading_stack: Vec<String>, // For cycle detection
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            search_path: ModuleSearchPath::new(),
            loaded_modules: HashMap::new(),
            loading_stack: Vec::new(),
        }
    }
    
    pub fn with_search_path(search_path: ModuleSearchPath) -> Self {
        Self {
            search_path,
            loaded_modules: HashMap::new(),
            loading_stack: Vec::new(),
        }
    }
    
    /// Add a search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_path.add_path(path);
    }
    
    /// Check if a module is already loaded
    pub fn is_loaded(&self, module_name: &str) -> bool {
        self.loaded_modules.contains_key(module_name)
    }
    
    /// Check if currently loading a module (cycle detection)
    pub fn is_loading(&self, module_name: &str) -> bool {
        self.loading_stack.contains(&module_name.to_string())
    }
    
    /// Load a module by name
    pub fn load_module(&mut self, module_name: &str) -> Result<&LoadedModule, String> {
        // Check if already loaded
        if self.is_loaded(module_name) {
            return Ok(self.loaded_modules.get(module_name).unwrap());
        }
        
        // Check for circular imports
        if self.is_loading(module_name) {
            return Err(format!(
                "Circular import detected: {} -> {}",
                self.loading_stack.join(" -> "),
                module_name
            ));
        }
        
        // Find module file
        let module_path = self.search_path.find_module(module_name)
            .ok_or_else(|| format!("Module '{}' not found. Searched in: {:?}", 
                module_name, 
                self.search_path.paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
            ))?;
        
        self.loading_stack.push(module_name.to_string());

        // Read and parse module
        let source = fs::read_to_string(&module_path)
            .map_err(|e| format!("Failed to read module '{}': {}", module_path.display(), e))?;

        // Tokenize first
        let mut lexer = crate::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize()
            .map_err(|e| format!("Failed to tokenize module '{}': {}", module_path.display(), e))?;

        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| format!("Failed to parse module '{}': {}", module_path.display(), e))?;
        
        // Process imports in the module first
        let import_result = self.process_module_imports(&ast, &module_path);
        self.loading_stack.pop();
        import_result?;
        
        // Store loaded module
        let loaded = LoadedModule {
            name: module_name.to_string(),
            path: module_path,
            ast,
            initialized: false,
        };
        
        self.loaded_modules.insert(module_name.to_string(), loaded);
        
        Ok(self.loaded_modules.get(module_name).unwrap())
    }
    
    /// Process imports in a module
    fn process_module_imports(&mut self, module: &Module, module_path: &Path) -> Result<(), String> {
        // Collect module names to import first (to avoid borrow issues)
        let mut modules_to_import: Vec<String> = Vec::new();
        
        for stmt in &module.statements {
            match stmt {
                crate::ast::Stmt::Import { module: mod_name, .. } => {
                    modules_to_import.push(mod_name.clone());
                }
                crate::ast::Stmt::FromImport { module: mod_name, .. } => {
                    modules_to_import.push(mod_name.clone());
                }
                _ => {}
            }
        }
        
        // Now load the modules
        for mod_name in modules_to_import {
            // Resolve relative to current module's directory
            if let Some(parent) = module_path.parent() {
                self.search_path.add_path(parent.to_path_buf());
                self.load_module(&mod_name)?;
            } else {
                self.load_module(&mod_name)?;
            }
        }
        Ok(())
    }
    
    /// Get a loaded module
    pub fn get_module(&self, module_name: &str) -> Option<&LoadedModule> {
        self.loaded_modules.get(module_name)
    }
    
    /// Get mutable reference to loaded module
    pub fn get_module_mut(&mut self, module_name: &str) -> Option<&mut LoadedModule> {
        self.loaded_modules.get_mut(module_name)
    }
    
    /// Mark a module as initialized
    pub fn mark_initialized(&mut self, module_name: &str) {
        if let Some(module) = self.loaded_modules.get_mut(module_name) {
            module.initialized = true;
        }
    }
    
    /// Get all loaded modules
    pub fn loaded_modules(&self) -> &HashMap<String, LoadedModule> {
        &self.loaded_modules
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}
