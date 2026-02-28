#![allow(dead_code)]
/// Viper type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 8-bit integer
    I8,
    /// 16-bit integer
    I16,
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// Auto-promoting integer (tagged: small int or BigInt on overflow)
    Int,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Boolean
    Bool,
    /// String
    Str,
    /// Bytes (immutable byte sequence)
    Bytes,
    /// BigInt (arbitrary precision integer using GMP)
    BigInt,
    /// Unit/None type
    None,
    /// List of elements (dynamic size)
    List(Box<Type>),
    /// Dictionary (key-value pairs)
    Dict(Box<Type>, Box<Type>),
    /// Tuple (fixed-size heterogeneous)
    Tuple(Vec<Type>),
    /// Array (fixed-size homogeneous collection)
    Array(Box<Type>, usize),
    /// Function type
    Fn(Vec<Type>, Box<Type>),
    /// Channel type (for concurrency)
    Chan(Box<Type>),
    /// WaitGroup type (for synchronization)
    WaitGroup,
    /// Optional type (nullable)
    Optional(Box<Type>),
    /// Struct type
    Struct { name: String, fields: Vec<(String, Type)> },
    /// Future type (for async/await)
    Future(Box<Type>),
    /// Type variable (for generics)
    Var(String),
    /// Unknown/to be inferred
    Infer,
    /// Error type
    Error,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F32 | Type::F64 | Type::BigInt)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::BigInt)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self, Type::Infer)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    /// Check if a type is hashable (can be used as dict key or set element)
    ///
    /// Hashable types:
    /// - int, float, bool, str, bytes
    /// - tuple (if all elements are hashable)
    /// - frozenset
    ///
    /// Non-hashable types:
    /// - list, dict, set
    /// - mutable custom objects
    pub fn is_hashable(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::F32
                | Type::F64
                | Type::Bool
                | Type::Str
                | Type::Bytes
                | Type::BigInt
        )
    }

    /// Check if a type is a tuple with all hashable elements
    pub fn is_hashable_tuple(&self) -> bool {
        match self {
            Type::Tuple(types) => types.iter().all(|t| t.is_hashable() || t.is_hashable_tuple()),
            _ => false,
        }
    }

    /// Check if a type is completely hashable (including nested structures)
    pub fn is_fully_hashable(&self) -> bool {
        match self {
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Str
            | Type::BigInt => true,
            Type::Tuple(types) => types.iter().all(|t| t.is_fully_hashable()),
            // List, Dict, Array are not hashable
            Type::List(_) | Type::Dict(_, _) | Type::Array(_, _) => false,
            // Other types are not hashable by default
            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Bytes => write!(f, "bytes"),
            Type::BigInt => write!(f, "BigInt"),
            Type::None => write!(f, "None"),
            Type::Int => write!(f, "int"),
            Type::List(t) => write!(f, "[{}]", t),
            Type::Dict(k, v) => write!(f, "{{{}: {}}}", k, v),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Array(elem_type, size) => write!(f, "[{}; {}]", elem_type, size),
            Type::Fn(params, ret) => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Chan(t) => write!(f, "chan[{}]", t),
            Type::WaitGroup => write!(f, "WaitGroup"),
            Type::Optional(t) => write!(f, "{}?", t),
            Type::Struct { name, fields } => {
                write!(f, "struct {} {{ ", name)?;
                for (i, (field_name, field_type)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field_name, field_type)?;
                }
                write!(f, " }}")
            }
            Type::Future(t) => write!(f, "Future[{}]", t),
            Type::Var(name) => write!(f, "{}", name),
            Type::Infer => write!(f, "_"),
            Type::Error => write!(f, "<error>"),
        }
    }
}
