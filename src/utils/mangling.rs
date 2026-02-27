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

fn mangle_type(ty: &Type) -> String {
    match ty {
        Type::I8 => "i8".to_string(),
        Type::I16 => "i16".to_string(),
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::BigInt => "bigint".to_string(),
        Type::F32 => "f32".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Str => "str".to_string(),
        Type::Bytes => "bytes".to_string(),
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
        Type::Infer => "infer".to_string(),
        Type::Error => "error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mangle_no_params() {
        assert_eq!(mangle_function_name("foo", &[]), "foo");
    }

    #[test]
    fn test_mangle_single_param() {
        assert_eq!(mangle_function_name("foo", &[Type::I64]), "foo_i64");
    }

    #[test]
    fn test_mangle_multiple_params() {
        assert_eq!(mangle_function_name("add", &[Type::I64, Type::I64]), "add_i64_i64");
        assert_eq!(mangle_function_name("foo", &[Type::I64, Type::F64]), "foo_i64_f64");
    }
}
