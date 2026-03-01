//! Tests for semantic::symbol_table module

use viper_lang::semantic::{Symbol, SymbolKind, SymbolTable};
use viper_lang::semantic::symbol_table::BuiltinSignature;
use viper_lang::ast::Type;
use viper_lang::utils::Span;

#[test]
fn test_symbol_table_new() {
    let table = SymbolTable::new();
    assert_eq!(table.current_scope_id(), 0);
    // Builtins should be inserted
    assert!(table.contains("print"));
    assert!(table.contains("len"));
}

#[test]
fn test_insert_and_lookup() {
    let mut table = SymbolTable::new();
    let span = Span::empty(1, 0);
    
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) },
        span,
        0
    );
    
    table.insert(symbol);
    
    let found = table.lookup("x");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "x");
}

#[test]
fn test_lookup_not_found() {
    let table = SymbolTable::new();
    assert!(table.lookup("nonexistent").is_none());
}

#[test]
fn test_enter_exit_scope() {
    let mut table = SymbolTable::new();
    
    // Initially in scope 0
    assert_eq!(table.current_scope_id(), 0);
    
    // Enter new scope
    let scope1 = table.enter_scope();
    assert_eq!(scope1, 1);
    assert_eq!(table.current_scope_id(), 1);
    
    // Enter another scope
    let scope2 = table.enter_scope();
    assert_eq!(scope2, 2);
    assert_eq!(table.current_scope_id(), 2);
    
    // Exit scope
    table.exit_scope();
    assert_eq!(table.current_scope_id(), 1);
    
    // Exit again
    table.exit_scope();
    assert_eq!(table.current_scope_id(), 0);
}

#[test]
fn test_scope_isolation() {
    let mut table = SymbolTable::new();
    let span = Span::empty(1, 0);
    
    // Insert in global scope
    let global_sym = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) },
        span,
        0
    );
    table.insert(global_sym);
    
    // Enter new scope
    table.enter_scope();
    
    // Insert different variable with same name in inner scope
    let inner_sym = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::Str) },
        span,
        1
    );
    table.insert(inner_sym);
    
    // Lookup should find inner scope version
    let found = table.lookup("x").unwrap();
    assert!(matches!(found.kind, SymbolKind::Variable { type_ann: Some(Type::Str), .. }));
    
    // Exit scope
    table.exit_scope();
    
    // Now should find global version
    let found = table.lookup("x").unwrap();
    assert!(matches!(found.kind, SymbolKind::Variable { type_ann: Some(Type::I64), .. }));
}

#[test]
fn test_lookup_from_inner_scope() {
    let mut table = SymbolTable::new();
    let span = Span::empty(1, 0);
    
    // Insert in global scope
    let global_sym = Symbol::new(
        "global_var".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) },
        span,
        0
    );
    table.insert(global_sym);
    
    // Enter new scope
    table.enter_scope();
    
    // Should still find global variable
    let found = table.lookup("global_var");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "global_var");
}

#[test]
fn test_contains() {
    let mut table = SymbolTable::new();
    let span = Span::empty(1, 0);
    
    assert!(!table.contains("myvar"));
    
    let symbol = Symbol::new(
        "myvar".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: None },
        span,
        0
    );
    table.insert(symbol);
    
    assert!(table.contains("myvar"));
}

#[test]
fn test_get_type_variable() {
    let span = Span::empty(1, 0);
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) },
        span,
        0
    );
    
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_get_type_function() {
    let span = Span::empty(1, 0);
    let symbol = Symbol::new_function(
        "add".to_string(),
        vec![Type::I64, Type::I64],
        Some(Type::I64),
        span,
        0,
        vec![]
    );
    
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_get_type_builtin() {
    let span = Span::empty(1, 0);
    let symbol = Symbol::new(
        "len".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Len },
        span,
        0
    );
    
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_symbol_clone() {
    let span = Span::empty(1, 0);
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) },
        span,
        0
    );
    
    let cloned = symbol.clone();
    assert_eq!(symbol.name, cloned.name);
}

#[test]
fn test_default_symbol_table() {
    let table: SymbolTable = Default::default();
    assert_eq!(table.current_scope_id(), 0);
    // Builtins should be inserted
    assert!(table.contains("print"));
}

#[test]
fn test_get_all_symbols() {
    let table = SymbolTable::new();
    let all = table.get_all_symbols();
    // Should have builtins
    assert!(!all.is_empty());
    assert!(all.iter().any(|(name, _)| *name == "print"));
}

#[test]
fn test_resolve_type_alias() {
    let mut table = SymbolTable::new();
    let span = Span::empty(1, 0);
    
    // Add type alias
    let alias_sym = Symbol::new(
        "MyInt".to_string(),
        SymbolKind::TypeAlias { type_def: Type::I64 },
        span,
        0
    );
    table.insert(alias_sym);
    
    // Resolve type alias
    let resolved = table.resolve_type_alias(&Type::Var("MyInt".to_string()));
    assert_eq!(resolved, Type::I64);
}

#[test]
fn test_lookup_mut() {
    let mut table = SymbolTable::new();
    let span = Span::empty(1, 0);
    
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) },
        span,
        0
    );
    table.insert(symbol);
    
    // Modify through mutable lookup
    if let Some(sym) = table.lookup_mut("x") {
        sym.kind = SymbolKind::Variable { mutable: true, type_ann: Some(Type::Str) };
    }
    
    let found = table.lookup("x").unwrap();
    assert!(matches!(found.kind, SymbolKind::Variable { type_ann: Some(Type::Str), .. }));
}
