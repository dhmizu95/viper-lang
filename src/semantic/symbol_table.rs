use crate::ast::Type;
use crate::utils::Span;
use std::collections::HashMap;

/// Kind of symbol (variable, function, parameter)
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable { mutable: bool, type_ann: Option<Type> },
    Function { params: Vec<Type>, return_type: Option<Type> },
    Parameter { type_ann: Option<Type> },
    Builtin { signature: BuiltinSignature },
}

/// Built-in function signatures
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinSignature {
    Print,
    Range,
    Len,
    Str,
    Int,
    Float,
    Bool,
    List,
    Append,
    Insert,
    Remove,
    Pop,
    Clear,
    Index,
}

/// A symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    pub scope_id: usize,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, span: Span, scope_id: usize) -> Self {
        Self { name, kind, span, scope_id }
    }

    pub fn get_type(&self) -> Option<Type> {
        match &self.kind {
            SymbolKind::Variable { type_ann, .. } => type_ann.clone(),
            SymbolKind::Function { return_type, .. } => return_type.clone(),
            SymbolKind::Parameter { type_ann } => type_ann.clone(),
            SymbolKind::Builtin { signature } => {
                match signature {
                    BuiltinSignature::Print => Some(Type::None),
                    BuiltinSignature::Range => Some(Type::List(Box::new(Type::I64))),
                    BuiltinSignature::Len => Some(Type::I64),
                    BuiltinSignature::Str => Some(Type::Str),
                    BuiltinSignature::Int => Some(Type::I64),
                    BuiltinSignature::Float => Some(Type::F64),
                    BuiltinSignature::Bool => Some(Type::Bool),
                    BuiltinSignature::List => Some(Type::List(Box::new(Type::Infer))),
                    BuiltinSignature::Append => Some(Type::None),
                    BuiltinSignature::Insert => Some(Type::None),
                    BuiltinSignature::Remove => Some(Type::None),
                    BuiltinSignature::Pop => Some(Type::Infer),
                    BuiltinSignature::Clear => Some(Type::None),
                    BuiltinSignature::Index => Some(Type::Infer),
                }
            }
        }
    }
}

/// Symbol table with scope support
#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    current_scope: usize,
    scope_chain: Vec<usize>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            scopes: vec![HashMap::new()],
            current_scope: 0,
            scope_chain: vec![0],
        };
        table.insert_builtins();
        table
    }

    /// Insert built-in functions
    fn insert_builtins(&mut self) {
        let builtins = vec![
            ("print", SymbolKind::Builtin { signature: BuiltinSignature::Print }),
            ("range", SymbolKind::Builtin { signature: BuiltinSignature::Range }),
            ("len", SymbolKind::Builtin { signature: BuiltinSignature::Len }),
            ("str", SymbolKind::Builtin { signature: BuiltinSignature::Str }),
            ("int", SymbolKind::Builtin { signature: BuiltinSignature::Int }),
            ("float", SymbolKind::Builtin { signature: BuiltinSignature::Float }),
            ("bool", SymbolKind::Builtin { signature: BuiltinSignature::Bool }),
            ("list", SymbolKind::Builtin { signature: BuiltinSignature::List }),
        ];

        let span = Span::empty(0, 0);
        for (name, kind) in builtins {
            let symbol = Symbol::new(name.to_string(), kind, span, 0);
            self.scopes[0].insert(name.to_string(), symbol);
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) -> usize {
        let new_scope_id = self.scopes.len();
        self.scopes.push(HashMap::new());
        self.current_scope = new_scope_id;
        self.scope_chain.push(new_scope_id);
        new_scope_id
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if self.scope_chain.len() > 1 {
            self.scope_chain.pop();
            self.current_scope = *self.scope_chain.last().unwrap();
        }
    }

    /// Insert a symbol into current scope
    pub fn insert(&mut self, symbol: Symbol) -> Result<(), String> {
        let scope = &mut self.scopes[self.current_scope];
        if scope.contains_key(&symbol.name) {
            return Err(format!("'{}' is already defined", symbol.name));
        }
        scope.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol by name
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for &scope_id in self.scope_chain.iter().rev() {
            if let Some(symbol) = self.scopes[scope_id].get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Look up a symbol mutably
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for &scope_id in self.scope_chain.iter().rev() {
            if self.scopes[scope_id].contains_key(name) {
                return self.scopes[scope_id].get_mut(name);
            }
        }
        None
    }

    /// Check if a symbol exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Get current scope ID
    pub fn current_scope_id(&self) -> usize {
        self.current_scope
    }

    /// Get all symbols in current scope
    pub fn get_current_scope_symbols(&self) -> Vec<&Symbol> {
        self.scopes[self.current_scope].values().collect()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
