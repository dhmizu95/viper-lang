use crate::ast::Type;
use crate::utils::mangle_function_name;
use crate::utils::Span;
use std::collections::HashMap;

/// Kind of symbol (variable, function, parameter)
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable { mutable: bool, type_ann: Option<Type> },
    /// Function with optional generic type parameters
    Function { 
        params: Vec<Type>, 
        return_type: Option<Type>, 
        mangled_name: String,
        type_params: Vec<String>,  // Generic type parameter names (e.g., ["T", "U"])
    },
    Parameter { type_ann: Option<Type> },
    Builtin { signature: BuiltinSignature },
    TypeAlias { type_def: Type },
    /// Generic type definition (e.g., class MyList[T])
    GenericTypeDef { type_params: Vec<String> },
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
    Reserve,
    Insert,
    Remove,
    Pop,
    Clear,
    Index,
    Hash,
    // Concurrency primitives (Phase 3)
    ChanCreate,      // chan(capacity) -> Chan[T]
    ChanSend,        // send(chan, value) -> None
    ChanRecv,        // recv(chan) -> T
    WaitGroupCreate, // WaitGroup() -> WaitGroup
    WaitGroupAdd,    // add(wg, n) -> None
    WaitGroupDone,   // done(wg) -> None
    WaitGroupWait,   // wait(wg) -> None
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

    pub fn new_function(
        name: String,
        params: Vec<Type>,
        return_type: Option<Type>,
        span: Span,
        scope_id: usize,
        type_params: Vec<String>,
    ) -> Self {
        let mangled_name = mangle_function_name(&name, &params);
        Self {
            name,
            kind: SymbolKind::Function { params, return_type, mangled_name, type_params },
            span,
            scope_id,
        }
    }

    pub fn get_type(&self) -> Option<Type> {
        match &self.kind {
            SymbolKind::Variable { type_ann, .. } => type_ann.clone(),
            SymbolKind::Function { return_type, .. } => return_type.clone(),
            SymbolKind::Parameter { type_ann } => type_ann.clone(),
            SymbolKind::Builtin { signature } => match signature {
                BuiltinSignature::Print => Some(Type::None),
                BuiltinSignature::Range => Some(Type::List(Box::new(Type::I64))),
                BuiltinSignature::Len => Some(Type::I64),
                BuiltinSignature::Str => Some(Type::Str),
                BuiltinSignature::Int => Some(Type::I64),
                BuiltinSignature::Float => Some(Type::F64),
                BuiltinSignature::Bool => Some(Type::Bool),
                BuiltinSignature::List => Some(Type::List(Box::new(Type::Infer))),
                BuiltinSignature::Append => Some(Type::None),
                BuiltinSignature::Reserve => Some(Type::None),
                BuiltinSignature::Insert => Some(Type::None),
                BuiltinSignature::Remove => Some(Type::None),
                BuiltinSignature::Pop => Some(Type::Infer),
                BuiltinSignature::Clear => Some(Type::None),
                BuiltinSignature::Index => Some(Type::Infer),
                BuiltinSignature::Hash => Some(Type::I64),
                // Concurrency primitives return pointer types
                BuiltinSignature::ChanCreate => Some(Type::Infer), // Chan[T] - element type inferred from usage
                BuiltinSignature::ChanSend => Some(Type::None),
                BuiltinSignature::ChanRecv => Some(Type::Infer), // Returns channel element type
                BuiltinSignature::WaitGroupCreate => Some(Type::WaitGroup),
                BuiltinSignature::WaitGroupAdd => Some(Type::None),
                BuiltinSignature::WaitGroupDone => Some(Type::None),
                BuiltinSignature::WaitGroupWait => Some(Type::None),
            },
            SymbolKind::TypeAlias { type_def } => Some(type_def.clone()),
            SymbolKind::GenericTypeDef { .. } => None,
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
        let mut table =
            Self { scopes: vec![HashMap::new()], current_scope: 0, scope_chain: vec![0] };
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
            ("hash", SymbolKind::Builtin { signature: BuiltinSignature::Hash }),
            // Concurrency builtins (Phase 3)
            ("chan", SymbolKind::Builtin { signature: BuiltinSignature::ChanCreate }),
            ("send", SymbolKind::Builtin { signature: BuiltinSignature::ChanSend }),
            ("recv", SymbolKind::Builtin { signature: BuiltinSignature::ChanRecv }),
            ("WaitGroup", SymbolKind::Builtin { signature: BuiltinSignature::WaitGroupCreate }),
            ("add", SymbolKind::Builtin { signature: BuiltinSignature::WaitGroupAdd }),
            ("done", SymbolKind::Builtin { signature: BuiltinSignature::WaitGroupDone }),
            ("wait", SymbolKind::Builtin { signature: BuiltinSignature::WaitGroupWait }),
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
    /// For functions, allows overloading by using mangled name
    pub fn insert(&mut self, symbol: Symbol) -> Result<(), String> {
        let scope = &mut self.scopes[self.current_scope];

        if let SymbolKind::Function { mangled_name, .. } = &symbol.kind {
            let key = mangled_name.clone();
            scope.insert(key, symbol);
        } else {
            if scope.contains_key(&symbol.name) {
                return Err(format!("'{}' is already defined", symbol.name));
            }
            scope.insert(symbol.name.clone(), symbol);
        }
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

    /// Look up a function by its mangled name
    pub fn lookup_mangled(&self, mangled_name: &str) -> Option<&Symbol> {
        for &scope_id in self.scope_chain.iter().rev() {
            if let Some(symbol) = self.scopes[scope_id].get(mangled_name) {
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

    /// Resolve a type alias to its underlying type
    pub fn resolve_type_alias(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(name) => {
                // Check if this is a type alias
                if let Some(symbol) = self.lookup(name) {
                    if let SymbolKind::TypeAlias { type_def } = &symbol.kind {
                        // Recursively resolve in case of nested aliases
                        return self.resolve_type_alias(type_def);
                    }
                }
                ty.clone()
            }
            Type::List(inner) => Type::List(Box::new(self.resolve_type_alias(inner))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.resolve_type_alias(k)),
                Box::new(self.resolve_type_alias(v)),
            ),
            Type::Optional(inner) => Type::Optional(Box::new(self.resolve_type_alias(inner))),
            Type::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.resolve_type_alias(t)).collect())
            }
            Type::Array(elem, size) => Type::Array(Box::new(self.resolve_type_alias(elem)), *size),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|t| self.resolve_type_alias(t)).collect(),
                Box::new(self.resolve_type_alias(ret)),
            ),
            Type::Chan(inner) => Type::Chan(Box::new(self.resolve_type_alias(inner))),
            Type::Future(inner) => Type::Future(Box::new(self.resolve_type_alias(inner))),
            Type::GenericApp { name, type_args } => Type::GenericApp {
                name: name.clone(),
                type_args: type_args.iter().map(|t| self.resolve_type_alias(t)).collect(),
            },
            Type::Union(variants) => Type::Union(
                variants.iter().map(|t| self.resolve_type_alias(t)).collect(),
            ),
            _ => ty.clone(),
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
