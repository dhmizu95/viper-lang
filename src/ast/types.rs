#![allow(dead_code)]
/// Viper type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 64-bit integer
    I64,
    /// 64-bit float
    F64,
    /// Boolean
    Bool,
    /// String
    Str,
    /// Unit/None type
    None,
    /// List of elements
    List(Box<Type>),
    /// Function type
    Fn(Vec<Type>, Box<Type>),
    /// Type variable (for generics)
    Var(String),
    /// Unknown/to be inferred
    Infer,
    /// Error type
    Error,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::I64 | Type::F64)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self, Type::Infer)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I64 => write!(f, "i64"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::None => write!(f, "None"),
            Type::List(t) => write!(f, "[{}]", t),
            Type::Fn(params, ret) => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Var(name) => write!(f, "{}", name),
            Type::Infer => write!(f, "_"),
            Type::Error => write!(f, "<error>"),
        }
    }
}
