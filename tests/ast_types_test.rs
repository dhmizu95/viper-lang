//! Tests for ast::types module

use viper_lang::ast::Type;

#[test]
fn test_is_numeric() {
    assert!(Type::I8.is_numeric());
    assert!(Type::I16.is_numeric());
    assert!(Type::I32.is_numeric());
    assert!(Type::I64.is_numeric());
    assert!(Type::F32.is_numeric());
    assert!(Type::F64.is_numeric());
    assert!(Type::BigInt.is_numeric());
    assert!(!Type::Str.is_numeric());
    assert!(!Type::Bool.is_numeric());
    assert!(!Type::None.is_numeric());
}

#[test]
fn test_is_integer() {
    assert!(Type::I8.is_integer());
    assert!(Type::I16.is_integer());
    assert!(Type::I32.is_integer());
    assert!(Type::I64.is_integer());
    assert!(Type::BigInt.is_integer());
    assert!(!Type::F32.is_integer());
    assert!(!Type::F64.is_integer());
    assert!(!Type::Str.is_integer());
}

#[test]
fn test_is_float() {
    assert!(Type::F32.is_float());
    assert!(Type::F64.is_float());
    assert!(!Type::I64.is_float());
    assert!(!Type::Str.is_float());
}

#[test]
fn test_is_hashable() {
    assert!(Type::I64.is_hashable());
    assert!(Type::F64.is_hashable());
    assert!(Type::Bool.is_hashable());
    assert!(Type::Str.is_hashable());
    assert!(Type::Bytes.is_hashable());
    assert!(Type::BigInt.is_hashable());
    assert!(!Type::List(Box::new(Type::I64)).is_hashable());
    assert!(!Type::Dict(Box::new(Type::Str), Box::new(Type::I64)).is_hashable());
}

#[test]
fn test_is_fully_hashable() {
    assert!(Type::I64.is_fully_hashable());
    assert!(Type::Str.is_fully_hashable());
    
    // Tuple with all hashable elements
    let hashable_tuple = Type::Tuple(vec![Type::I64, Type::Str]);
    assert!(hashable_tuple.is_fully_hashable());
    
    // List is not hashable
    assert!(!Type::List(Box::new(Type::I64)).is_fully_hashable());
    
    // Dict is not hashable
    assert!(!Type::Dict(Box::new(Type::Str), Box::new(Type::I64)).is_fully_hashable());
}

#[test]
fn test_is_union() {
    let union = Type::Union(vec![Type::I64, Type::Str]);
    assert!(union.is_union());
    assert!(!Type::I64.is_union());
}

#[test]
fn test_union_variants() {
    let union = Type::Union(vec![Type::I64, Type::Str, Type::Bool]);
    let variants = union.union_variants().unwrap();
    assert_eq!(variants.len(), 3);
    assert!(variants.contains(&Type::I64));
    assert!(variants.contains(&Type::Str));
    assert!(variants.contains(&Type::Bool));
    
    assert!(Type::I64.union_variants().is_none());
}

#[test]
fn test_is_in_union() {
    let union = Type::Union(vec![Type::I64, Type::Str]);
    assert!(Type::I64.is_in_union(&union));
    assert!(Type::Str.is_in_union(&union));
    assert!(!Type::Bool.is_in_union(&union));
}

#[test]
fn test_type_equality() {
    let t1 = Type::I64;
    let t2 = Type::I64;
    let t3 = Type::Str;
    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

#[test]
fn test_type_clone() {
    let t1 = Type::List(Box::new(Type::I64));
    let t2 = t1.clone();
    assert_eq!(t1, t2);
}

#[test]
fn test_list_type() {
    let list = Type::List(Box::new(Type::I64));
    assert!(!list.is_hashable());
}

#[test]
fn test_dict_type() {
    let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::I64));
    if let Type::Dict(k, v) = dict {
        assert_eq!(*k, Type::Str);
        assert_eq!(*v, Type::I64);
    } else {
        panic!("Expected Dict type");
    }
}

#[test]
fn test_tuple_type() {
    let tuple = Type::Tuple(vec![Type::I64, Type::Str, Type::Bool]);
    if let Type::Tuple(types) = tuple {
        assert_eq!(types.len(), 3);
        assert_eq!(types[0], Type::I64);
        assert_eq!(types[1], Type::Str);
        assert_eq!(types[2], Type::Bool);
    } else {
        panic!("Expected Tuple type");
    }
}

#[test]
fn test_optional_type() {
    let opt = Type::Optional(Box::new(Type::I64));
    if let Type::Optional(inner) = opt {
        assert_eq!(*inner, Type::I64);
    } else {
        panic!("Expected Optional type");
    }
}

#[test]
fn test_result_type() {
    let result = Type::Result(Box::new(Type::I64), Box::new(Type::Str));
    if let Type::Result(ok, err) = result {
        assert_eq!(*ok, Type::I64);
        assert_eq!(*err, Type::Str);
    } else {
        panic!("Expected Result type");
    }
}

#[test]
fn test_function_type() {
    let func = Type::Fn(vec![Type::I64, Type::I64], Box::new(Type::I64));
    if let Type::Fn(params, ret) = func {
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], Type::I64);
        assert_eq!(*ret, Type::I64);
    } else {
        panic!("Expected Fn type");
    }
}

#[test]
fn test_channel_type() {
    let chan = Type::Chan(Box::new(Type::I64));
    if let Type::Chan(inner) = chan {
        assert_eq!(*inner, Type::I64);
    } else {
        panic!("Expected Chan type");
    }
}

#[test]
fn test_struct_type() {
    let struct_type = Type::Struct {
        name: "Point".to_string(),
        fields: vec![("x".to_string(), Type::I64), ("y".to_string(), Type::I64)],
    };
    if let Type::Struct { name, fields } = struct_type {
        assert_eq!(name, "Point");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "x");
        assert_eq!(fields[0].1, Type::I64);
    } else {
        panic!("Expected Struct type");
    }
}

#[test]
fn test_array_type() {
    let array = Type::Array(Box::new(Type::I64), 10);
    if let Type::Array(inner, size) = array {
        assert_eq!(*inner, Type::I64);
        assert_eq!(size, 10);
    } else {
        panic!("Expected Array type");
    }
}

#[test]
fn test_generic_app() {
    let generic = Type::GenericApp { 
        name: "List".to_string(), 
        type_args: vec![Type::I64] 
    };
    assert!(generic.is_generic_app());
    let (name, args) = generic.as_generic_app().unwrap();
    assert_eq!(name, "List");
    assert_eq!(args.len(), 1);
}
