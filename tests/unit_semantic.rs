//! Unit tests for the Viper semantic module
//! Tests for: SymbolTable (new, enter_scope, exit_scope, insert, lookup, etc.)

use viper_lang::ast::Type;
use viper_lang::semantic::symbol_table::{
    BuiltinSignature, Symbol, SymbolKind, SymbolTable,
};
use viper_lang::utils::Span;

fn test_span() -> Span {
    Span::new(0, 5, 1, 1)
}

// ============================================================================
// SymbolTable::new Tests
// ============================================================================

#[test]
fn test_symbol_table_new_has_builtins() {
    let table = SymbolTable::new();
    
    // Check that built-in functions exist
    assert!(table.contains("print"));
    assert!(table.contains("range"));
    assert!(table.contains("len"));
    assert!(table.contains("str"));
    assert!(table.contains("int"));
    assert!(table.contains("float"));
    assert!(table.contains("bool"));
    assert!(table.contains("list"));
    assert!(table.contains("hash"));
}

#[test]
fn test_symbol_table_builtin_print() {
    let table = SymbolTable::new();
    let symbol = table.lookup("print").unwrap();
    assert!(matches!(symbol.kind, SymbolKind::Builtin { signature: BuiltinSignature::Print }));
}

#[test]
fn test_symbol_table_builtin_range() {
    let table = SymbolTable::new();
    let symbol = table.lookup("range").unwrap();
    assert!(matches!(symbol.kind, SymbolKind::Builtin { signature: BuiltinSignature::Range }));
}

#[test]
fn test_symbol_table_builtin_len() {
    let table = SymbolTable::new();
    let symbol = table.lookup("len").unwrap();
    assert!(matches!(symbol.kind, SymbolKind::Builtin { signature: BuiltinSignature::Len }));
}

#[test]
fn test_symbol_table_concurrency_builtins() {
    let table = SymbolTable::new();
    
    assert!(table.contains("chan"));
    assert!(table.contains("send"));
    assert!(table.contains("recv"));
    assert!(table.contains("WaitGroup"));
    assert!(table.contains("add"));
    assert!(table.contains("done"));
    assert!(table.contains("wait"));
}

// ============================================================================
// enter_scope / exit_scope Tests
// ============================================================================

#[test]
fn test_symbol_table_enter_scope() {
    let mut table = SymbolTable::new();
    let initial_scope = table.current_scope_id();
    
    let new_scope = table.enter_scope();
    
    assert_ne!(new_scope, initial_scope);
    assert_eq!(table.current_scope_id(), new_scope);
}

#[test]
fn test_symbol_table_exit_scope() {
    let mut table = SymbolTable::new();
    let initial_scope = table.current_scope_id();
    
    table.enter_scope();
    table.exit_scope();
    
    assert_eq!(table.current_scope_id(), initial_scope);
}

#[test]
fn test_symbol_table_multiple_scopes() {
    let mut table = SymbolTable::new();
    
    table.enter_scope();
    let scope1 = table.current_scope_id();
    
    table.enter_scope();
    let scope2 = table.current_scope_id();
    
    assert_ne!(scope1, scope2);
    assert!(scope2 > scope1);
    
    table.exit_scope();
    assert_eq!(table.current_scope_id(), scope1);
    
    table.exit_scope();
    assert_eq!(table.current_scope_id(), 0);
}

#[test]
fn test_symbol_table_exit_scope_restores_symbols() {
    let mut table = SymbolTable::new();
    
    // Insert in global scope
    let global_sym = Symbol::new(
        "global_var".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    table.insert(global_sym).unwrap();
    
    // Enter new scope and insert local var
    table.enter_scope();
    let local_sym = Symbol::new(
        "local_var".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        table.current_scope_id(),
    );
    table.insert(local_sym).unwrap();
    
    // Both should be visible
    assert!(table.contains("global_var"));
    assert!(table.contains("local_var"));
    
    // Exit scope
    table.exit_scope();
    
    // Global should still be visible, local should not
    assert!(table.contains("global_var"));
    assert!(!table.contains("local_var"));
}

// ============================================================================
// insert / lookup Tests
// ============================================================================

#[test]
fn test_symbol_table_insert_and_lookup() {
    let mut table = SymbolTable::new();
    
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        table.current_scope_id(),
    );
    
    table.insert(symbol.clone()).unwrap();
    
    let found = table.lookup("x").unwrap();
    assert_eq!(found.name, "x");
    assert_eq!(found.scope_id, 0);
}

#[test]
fn test_symbol_table_lookup_outer_scope() {
    let mut table = SymbolTable::new();
    
    // Insert in global scope
    let global_sym = Symbol::new(
        "outer".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    table.insert(global_sym).unwrap();
    
    // Enter inner scope
    table.enter_scope();
    
    // Should find outer scope symbol
    let found = table.lookup("outer").unwrap();
    assert_eq!(found.name, "outer");
}

#[test]
fn test_symbol_table_lookup_not_found() {
    let table = SymbolTable::new();
    let found = table.lookup("nonexistent");
    assert!(found.is_none());
}

#[test]
fn test_symbol_table_insert_function() {
    let mut table = SymbolTable::new();

    let symbol = Symbol::new_function(
        "foo".to_string(),
        vec![Type::I64, Type::I64],
        Some(Type::I64),
        test_span(),
        0,
        vec![],  // type_params
    );

    table.insert(symbol).unwrap();

    // Functions are stored by mangled name
    let found = table.lookup_mangled("foo_i64_i64").unwrap();
    assert_eq!(found.name, "foo");

    if let SymbolKind::Function { params, return_type, mangled_name, .. } = &found.kind {
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], Type::I64);
        assert_eq!(params[1], Type::I64);
        assert_eq!(*return_type, Some(Type::I64));
        assert_eq!(mangled_name, "foo_i64_i64");
    } else {
        panic!("Expected Function symbol");
    }
}

#[test]
fn test_symbol_table_duplicate_variable_insert() {
    let mut table = SymbolTable::new();
    
    let sym1 = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    table.insert(sym1).unwrap();
    
    let sym2 = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    
    // Duplicate insert should fail
    let result = table.insert(sym2);
    assert!(result.is_err());
}

#[test]
fn test_symbol_table_function_overloading() {
    let mut table = SymbolTable::new();

    // Insert foo(i64)
    let sym1 = Symbol::new_function(
        "foo".to_string(),
        vec![Type::I64],
        Some(Type::I64),
        test_span(),
        0,
        vec![],  // type_params
    );
    table.insert(sym1).unwrap();

    // Insert foo(f64) - should succeed (different mangled name)
    let sym2 = Symbol::new_function(
        "foo".to_string(),
        vec![Type::F64],
        Some(Type::F64),
        test_span(),
        0,
        vec![],  // type_params
    );
    let result = table.insert(sym2);
    assert!(result.is_ok());

    // Both should be findable by mangled name
    assert!(table.lookup_mangled("foo_i64").is_some());
    assert!(table.lookup_mangled("foo_f64").is_some());
}

#[test]
fn test_symbol_table_get_function_overloads() {
    let table = SymbolTable::new();

    // Manually insert some overloads
    let mut table_mut = table;
    
    let sym1 = Symbol::new_function(
        "bar".to_string(),
        vec![Type::I64],
        Some(Type::I64),
        test_span(),
        0,
        vec![],  // type_params
    );
    table_mut.insert(sym1).unwrap();
    
    let sym2 = Symbol::new_function(
        "bar".to_string(),
        vec![Type::F64],
        Some(Type::F64),
        test_span(),
        0,
        vec![],  // type_params
    );
    table_mut.insert(sym2).unwrap();
    
    let sym3 = Symbol::new_function(
        "bar".to_string(),
        vec![Type::Str],
        Some(Type::Str),
        test_span(),
        0,
        vec![],  // type_params
    );
    table_mut.insert(sym3).unwrap();

    // Get all overloads
    let overloads = table_mut.get_function_overloads("bar");
    assert_eq!(overloads.len(), 3);
    
    // Get overloads for non-existent function
    let empty = table_mut.get_function_overloads("nonexistent");
    assert_eq!(empty.len(), 0);
}

// ============================================================================
// contains Tests
// ============================================================================

#[test]
fn test_symbol_table_contains_existing() {
    let mut table = SymbolTable::new();
    
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    table.insert(symbol).unwrap();
    
    assert!(table.contains("x"));
}

#[test]
fn test_symbol_table_contains_not_existing() {
    let table = SymbolTable::new();
    assert!(!table.contains("nonexistent"));
}

#[test]
fn test_symbol_table_contains_after_exit_scope() {
    let mut table = SymbolTable::new();
    
    table.enter_scope();
    let symbol = Symbol::new(
        "inner".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        table.current_scope_id(),
    );
    table.insert(symbol).unwrap();
    
    assert!(table.contains("inner"));
    
    table.exit_scope();
    
    // Inner scope vars should not be visible
    assert!(!table.contains("inner"));
}

// ============================================================================
// lookup_mut Tests
// ============================================================================

#[test]
fn test_symbol_table_lookup_mut() {
    let mut table = SymbolTable::new();
    
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    table.insert(symbol).unwrap();
    
    let found = table.lookup_mut("x");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "x");
}

#[test]
fn test_symbol_table_lookup_mut_not_found() {
    let mut table = SymbolTable::new();
    let found = table.lookup_mut("nonexistent");
    assert!(found.is_none());
}

// ============================================================================
// get_current_scope_symbols Tests
// ============================================================================

#[test]
fn test_symbol_table_get_current_scope_symbols() {
    let mut table = SymbolTable::new();
    
    let sym1 = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    let sym2 = Symbol::new(
        "y".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    table.insert(sym1).unwrap();
    table.insert(sym2).unwrap();
    
    let symbols = table.get_current_scope_symbols();
    // 18 builtins + 2 inserted = 18 (builtins include print, range, etc.)
    // The actual count depends on how many builtins are defined
    assert!(symbols.len() >= 18); // At least the builtins
}

// ============================================================================
// resolve_type_alias Tests
// ============================================================================

#[test]
fn test_symbol_table_resolve_type_alias_single() {
    let mut table = SymbolTable::new();
    
    // Create a type alias: type IntAlias = i64
    let alias = Symbol::new(
        "IntAlias".to_string(),
        SymbolKind::TypeAlias { type_def: Type::I64 },
        test_span(),
        0,
    );
    table.insert(alias).unwrap();
    
    // Resolve the alias
    let resolved = table.resolve_type_alias(&Type::Var("IntAlias".to_string()));
    assert_eq!(resolved, Type::I64);
}

#[test]
fn test_symbol_table_resolve_type_alias_chained() {
    let mut table = SymbolTable::new();
    
    // Create chained aliases: type A = i64, type B = A
    let alias_a = Symbol::new(
        "A".to_string(),
        SymbolKind::TypeAlias { type_def: Type::I64 },
        test_span(),
        0,
    );
    table.insert(alias_a).unwrap();
    
    let alias_b = Symbol::new(
        "B".to_string(),
        SymbolKind::TypeAlias { type_def: Type::Var("A".to_string()) },
        test_span(),
        0,
    );
    table.insert(alias_b).unwrap();
    
    // Resolve B should give i64
    let resolved = table.resolve_type_alias(&Type::Var("B".to_string()));
    assert_eq!(resolved, Type::I64);
}

#[test]
fn test_symbol_table_resolve_type_alias_nested() {
    let mut table = SymbolTable::new();
    
    // type IntAlias = i64
    let alias = Symbol::new(
        "IntAlias".to_string(),
        SymbolKind::TypeAlias { type_def: Type::I64 },
        test_span(),
        0,
    );
    table.insert(alias).unwrap();
    
    // Resolve List[IntAlias] should give List[i64]
    let list_alias = Type::List(Box::new(Type::Var("IntAlias".to_string())));
    let resolved = table.resolve_type_alias(&list_alias);
    assert_eq!(resolved, Type::List(Box::new(Type::I64)));
}

#[test]
fn test_symbol_table_resolve_type_alias_non_existent() {
    let table = SymbolTable::new();
    
    // Non-existent type var should be returned as-is
    let unresolved = Type::Var("NonExistent".to_string());
    let resolved = table.resolve_type_alias(&unresolved);
    assert_eq!(resolved, Type::Var("NonExistent".to_string()));
}

// ============================================================================
// Symbol::get_type Tests
// ============================================================================

#[test]
fn test_symbol_get_type_variable() {
    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_symbol_get_type_function() {
    let symbol = Symbol::new_function(
        "foo".to_string(),
        vec![Type::I64],
        Some(Type::I64),
        test_span(),
        0,
        vec![],  // type_params
    );
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_symbol_get_type_builtin_print() {
    let symbol = Symbol::new(
        "print".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Print },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::None));
}

#[test]
fn test_symbol_get_type_builtin_range() {
    let symbol = Symbol::new(
        "range".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Range },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::List(Box::new(Type::I64))));
}

#[test]
fn test_symbol_get_type_builtin_len() {
    let symbol = Symbol::new(
        "len".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Len },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_symbol_get_type_builtin_str() {
    let symbol = Symbol::new(
        "str".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Str },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::Str));
}

#[test]
fn test_symbol_get_type_builtin_int() {
    let symbol = Symbol::new(
        "int".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Int },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

#[test]
fn test_symbol_get_type_builtin_float() {
    let symbol = Symbol::new(
        "float".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Float },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::F64));
}

#[test]
fn test_symbol_get_type_builtin_bool() {
    let symbol = Symbol::new(
        "bool".to_string(),
        SymbolKind::Builtin { signature: BuiltinSignature::Bool },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::Bool));
}

#[test]
fn test_symbol_get_type_type_alias() {
    let symbol = Symbol::new(
        "IntAlias".to_string(),
        SymbolKind::TypeAlias { type_def: Type::I64 },
        test_span(),
        0,
    );
    assert_eq!(symbol.get_type(), Some(Type::I64));
}

// ============================================================================
// SymbolKind Tests
// ============================================================================

#[test]
fn test_symbol_kind_variable_mutable() {
    let kind = SymbolKind::Variable { mutable: true, type_ann: Some(Type::I64) };
    match kind {
        SymbolKind::Variable { mutable, type_ann } => {
            assert!(mutable);
            assert_eq!(type_ann, Some(Type::I64));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_symbol_kind_variable_immutable() {
    let kind = SymbolKind::Variable { mutable: false, type_ann: Some(Type::I64) };
    match kind {
        SymbolKind::Variable { mutable, .. } => {
            assert!(!mutable);
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_symbol_kind_parameter() {
    let kind = SymbolKind::Parameter { type_ann: Some(Type::I64) };
    match kind {
        SymbolKind::Parameter { type_ann } => {
            assert_eq!(type_ann, Some(Type::I64));
        }
        _ => panic!("Expected Parameter"),
    }
}

// ============================================================================
// Union Type Tests
// ============================================================================

#[test]
fn test_union_type_display() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(format!("{}", union), "int | str");
}

#[test]
fn test_union_type_multiple_variants() {
    let union = Type::Union(vec![Type::I64, Type::Str, Type::Bool]);
    assert_eq!(format!("{}", union), "i64 | str | bool");
}

#[test]
fn test_union_type_is_union() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert!(union.is_union());
    
    let non_union = Type::I64;
    assert!(!non_union.is_union());
}

#[test]
fn test_union_type_variants() {
    let union = Type::Union(vec![Type::Int, Type::Str, Type::Bool]);
    let variants = union.union_variants().unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0], Type::Int);
    assert_eq!(variants[1], Type::Str);
    assert_eq!(variants[2], Type::Bool);
}

#[test]
fn test_union_type_is_in_union() {
    let union = Type::Union(vec![Type::Int, Type::Str, Type::Bool]);
    
    assert!(Type::Int.is_in_union(&union));
    assert!(Type::Str.is_in_union(&union));
    assert!(Type::Bool.is_in_union(&union));
    assert!(!Type::I64.is_in_union(&union));
}

#[test]
fn test_nested_union_type() {
    let nested = Type::Union(vec![
        Type::Int,
        Type::List(Box::new(Type::Union(vec![Type::I64, Type::Str])))
    ]);
    assert!(nested.is_union());
    // Nested union displays as: int | [i64 | str]
    assert_eq!(format!("{}", nested), "int | [i64 | str]");
}
