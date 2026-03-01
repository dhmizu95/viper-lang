use crate::ast::Type;

pub fn mangle_function_name(name: &str, param_types: &[Type]) -> String {
    if param_types.is_empty() {
        return name.to_string();
    }

    let mut mangled = String::new();
    mangled.push_str(name);
    mangled.push('_');

    for (i, ty) in param_types.iter().enumerate() {
        if i > 0 {
            mangled.push('_');
        }
        mangled.push_str(&mangle_type(ty));
    }

    mangled
}

/// Mangle function name with closure cell parameters appended
/// Format: name_type1_type2_..._closure_var1_var2_...
pub fn mangle_function_name_with_closure(
    name: &str,
    param_types: &[Type],
    nonlocal_vars: &[String],
) -> String {
    let base_mangled = mangle_function_name(name, param_types);
    
    if nonlocal_vars.is_empty() {
        return base_mangled;
    }
    
    let mut mangled = base_mangled;
    mangled.push_str("_closure");
    for var in nonlocal_vars {
        mangled.push('_');
        mangled.push_str(var);
    }
    
    mangled
}

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
        Type::Bytes => "bytes".to_string(),
        Type::BigInt => "bigint".to_string(),
        Type::Int => "int".to_string(),
        Type::None => "none".to_string(),
        Type::List(t) => format!("list_{}", mangle_type(t)),
        Type::Dict(k, v) => format!("dict_{}_{}", mangle_type(k), mangle_type(v)),
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(mangle_type).collect();
            format!("tuple{}", inner.join("_"))
        }
        Type::Array(t, size) => format!("array_{}_{}", mangle_type(t), size),
        Type::Fn(params, ret) => {
            let params_mangled: Vec<String> = params.iter().map(mangle_type).collect();
            format!("fn_{}_ret_{}", params_mangled.join("_"), mangle_type(ret))
        }
        Type::Chan(t) => format!("chan_{}", mangle_type(t)),
        Type::WaitGroup => "waitgroup".to_string(),
        Type::Optional(t) => format!("opt_{}", mangle_type(t)),
        Type::Struct { name, .. } => format!("struct_{}", name),
        Type::Future(t) => format!("future_{}", mangle_type(t)),
        Type::Var(name) => format!("var_{}", name),
        Type::TypeParam { name, bounds } => {
            if bounds.is_empty() {
                format!("typeparam_{}", name)
            } else {
                let bounds_mangled: Vec<String> = bounds.iter().map(mangle_type).collect();
                format!("typeparam_{}_bounds_{}", name, bounds_mangled.join("_"))
            }
        }
        Type::GenericApp { name, type_args } => {
            let args_mangled: Vec<String> = type_args.iter().map(mangle_type).collect();
            format!("generic_{}_{}", name, args_mangled.join("_"))
        }
        Type::Result(ok, err) => {
            format!("result_{}_{}", mangle_type(ok), mangle_type(err))
        }
        Type::Infer => "infer".to_string(),
        Type::Error => "error".to_string(),
        Type::Union(variants) => {
            let variants_mangled: Vec<String> = variants.iter().map(mangle_type).collect();
            format!("union_{}", variants_mangled.join("_"))
        }
        Type::Class(name) => format!("class_{}", name),
        Type::Instance(name) => format!("instance_{}", name),
        Type::Method { class_name, method_name, .. } => {
            format!("method_{}_{}", class_name, method_name)
        }
        Type::Object => "object".to_string(),
    }
}

