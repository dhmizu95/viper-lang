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
    /// Type variable (for generics) - unbound type parameter
    Var(String),
    /// Type parameter with bounds (for generics) - e.g., T: Hashable
    TypeParam { name: String, bounds: Vec<Type> },
    /// Generic type application - e.g., List[T], Dict[K, V], MyGeneric[T, U]
    GenericApp { name: String, type_args: Vec<Type> },
    /// Result type for error handling - Result[OkType, ErrType]
    Result(Box<Type>, Box<Type>),
    /// Unknown/to be inferred
    Infer,
    /// Error type
    Error,
    /// Union type (e.g., int | str)
    Union(Vec<Type>),
    /// Class type (reference to a class definition)
    Class(String),
    /// Instance of a class
    Instance(String),
    /// Method type (bound or unbound)
    Method {
        class_name: String,
        method_name: String,
        params: Vec<Type>,
        return_type: Box<Type>,
        is_bound: bool,  // true if 'self' is already bound
    },
    /// Base object type - root of class hierarchy
    /// Used for super() and as the common base for all class instances
    Object,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int | Type::F32 | Type::F64 | Type::BigInt)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int | Type::BigInt)
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
                | Type::Int
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
            | Type::Int
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Str
            | Type::BigInt => true,
            Type::Tuple(types) => types.iter().all(|t| t.is_fully_hashable()),
            // List, Dict, Array are not hashable
            Type::List(_) | Type::Dict(_, _) | Type::Array(_, _) => false,
            // Class instances are not hashable by default (mutable objects)
            Type::Instance(_) => false,
            // Other types are not hashable by default
            _ => false,
        }
    }

    /// Check if this is a union type
    pub fn is_union(&self) -> bool {
        matches!(self, Type::Union(_))
    }

    /// Get the variants of a union type, or empty vec if not a union
    pub fn union_variants(&self) -> Option<&Vec<Type>> {
        match self {
            Type::Union(variants) => Some(variants),
            _ => None,
        }
    }

    /// Check if this type is compatible with a union (i.e., is one of its variants)
    pub fn is_in_union(&self, union: &Type) -> bool {
        match union {
            Type::Union(variants) => variants.contains(self),
            _ => false,
        }
    }

    /// Check if this is a type parameter (unbound type variable)
    pub fn is_type_param(&self) -> bool {
        matches!(self, Type::Var(_) | Type::TypeParam { .. })
    }

    /// Get the name of a type parameter, if this is one
    pub fn as_type_param_name(&self) -> Option<&str> {
        match self {
            Type::Var(name) | Type::TypeParam { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Check if this is a generic type application
    pub fn is_generic_app(&self) -> bool {
        matches!(self, Type::GenericApp { .. })
    }

    /// Get the name and type arguments if this is a generic application
    pub fn as_generic_app(&self) -> Option<(&str, &[Type])> {
        match self {
            Type::GenericApp { name, type_args } => Some((name, type_args)),
            _ => None,
        }
    }

    /// Collect all type variables in this type
    pub fn collect_type_vars(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_type_vars_impl(&mut vars);
        vars
    }

    fn collect_type_vars_impl(&self, vars: &mut Vec<String>) {
        match self {
            Type::Var(name) | Type::TypeParam { name, .. } => {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            }
            Type::List(inner) => inner.collect_type_vars_impl(vars),
            Type::Dict(k, v) => {
                k.collect_type_vars_impl(vars);
                v.collect_type_vars_impl(vars);
            }
            Type::Tuple(types) | Type::Union(types) => {
                for t in types {
                    t.collect_type_vars_impl(vars);
                }
            }
            Type::Array(elem, _) => elem.collect_type_vars_impl(vars),
            Type::Fn(params, ret) => {
                for p in params {
                    p.collect_type_vars_impl(vars);
                }
                ret.collect_type_vars_impl(vars);
            }
            Type::Chan(inner) => inner.collect_type_vars_impl(vars),
            Type::Optional(inner) => inner.collect_type_vars_impl(vars),
            Type::Future(inner) => inner.collect_type_vars_impl(vars),
            Type::Struct { fields, .. } => {
                for (_, field_type) in fields {
                    field_type.collect_type_vars_impl(vars);
                }
            }
            Type::GenericApp { type_args, .. } => {
                for arg in type_args {
                    arg.collect_type_vars_impl(vars);
                }
            }
            Type::Result(ok, err) => {
                ok.collect_type_vars_impl(vars);
                err.collect_type_vars_impl(vars);
            }
            _ => {}
        }
    }

    /// Substitute type variables with concrete types
    /// 
    /// Given a type with type variables and a substitution map,
    /// replace all occurrences of type variables with their concrete types.
    pub fn substitute(&self, substitution: &std::collections::HashMap<String, Type>) -> Type {
        match self {
            Type::Var(name) => {
                substitution.get(name).cloned().unwrap_or_else(|| self.clone())
            }
            Type::TypeParam { name, bounds } => {
                if let Some(concrete) = substitution.get(name) {
                    concrete.clone()
                } else {
                    Type::TypeParam {
                        name: name.clone(),
                        bounds: bounds.iter().map(|b| b.substitute(substitution)).collect(),
                    }
                }
            }
            Type::List(inner) => Type::List(Box::new(inner.substitute(substitution))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(k.substitute(substitution)),
                Box::new(v.substitute(substitution)),
            ),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| t.substitute(substitution)).collect()),
            Type::Array(elem, size) => Type::Array(
                Box::new(elem.substitute(substitution)),
                *size,
            ),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| p.substitute(substitution)).collect(),
                Box::new(ret.substitute(substitution)),
            ),
            Type::Chan(inner) => Type::Chan(Box::new(inner.substitute(substitution))),
            Type::Optional(inner) => Type::Optional(Box::new(inner.substitute(substitution))),
            Type::Future(inner) => Type::Future(Box::new(inner.substitute(substitution))),
            Type::Struct { name, fields } => Type::Struct {
                name: name.clone(),
                fields: fields.iter().map(|(n, t)| (n.clone(), t.substitute(substitution))).collect(),
            },
            Type::GenericApp { name, type_args } => Type::GenericApp {
                name: name.clone(),
                type_args: type_args.iter().map(|t| t.substitute(substitution)).collect(),
            },
            Type::Result(ok, err) => Type::Result(
                Box::new(ok.substitute(substitution)),
                Box::new(err.substitute(substitution)),
            ),
            Type::Union(variants) => Type::Union(
                variants.iter().map(|t| t.substitute(substitution)).collect(),
            ),
            Type::Class(name) => Type::Class(name.clone()),
            Type::Instance(name) => Type::Instance(name.clone()),
            Type::Method { class_name, method_name, params, return_type, is_bound } => Type::Method {
                class_name: class_name.clone(),
                method_name: method_name.clone(),
                params: params.iter().map(|t| t.substitute(substitution)).collect(),
                return_type: Box::new(return_type.substitute(substitution)),
                is_bound: *is_bound,
            },
            // Primitive types and Infer/Error remain unchanged
            _ => self.clone(),
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
            Type::BigInt => write!(f, "int"),
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
            Type::TypeParam { name, bounds } => {
                write!(f, "{}", name)?;
                if !bounds.is_empty() {
                    write!(f, ": ")?;
                    for (i, bound) in bounds.iter().enumerate() {
                        if i > 0 {
                            write!(f, " + ")?;
                        }
                        write!(f, "{}", bound)?;
                    }
                }
                Ok(())
            }
            Type::GenericApp { name, type_args } => {
                write!(f, "{}[", name)?;
                for (i, arg) in type_args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, "]")
            }
            Type::Result(ok, err) => write!(f, "Result[{}, {}]", ok, err),
            Type::Infer => write!(f, "_"),
            Type::Error => write!(f, "<error>"),
            Type::Union(variants) => {
                for (i, variant) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", variant)?;
                }
                Ok(())
            }
            Type::Class(name) => write!(f, "class {}", name),
            Type::Instance(name) => write!(f, "{}", name),
            Type::Method { class_name, method_name, params, return_type, is_bound } => {
                if *is_bound {
                    write!(f, "method {}.{}(", class_name, method_name)?;
                } else {
                    write!(f, "unbound method {}.{}(", class_name, method_name)?;
                }
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Object => write!(f, "object"),
        }
    }
}

