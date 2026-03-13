use crate::ast::{Stmt, Type};
use crate::semantic::type_checker::{TypeChecker, TypeError};

impl TypeChecker {
    /// Check that all return statements are consistent with declared return type
    pub(crate) fn check_return_consistency(&mut self, body: &[Stmt], return_type: &Option<Type>) {
        // If no return type annotation, use Infer to allow any return type
        let expected = return_type.clone().unwrap_or(Type::Infer);

        // Simple check: find all return statements at any nesting level
        // In a real compiler, we would use a visitor pattern for this
        for stmt in body {
            match stmt {
                Stmt::Return { value, span } => {
                    let actual =
                        value.as_ref().and_then(|e| self.get_expr_type(e)).unwrap_or(Type::None);

                    if !self.is_compatible(&expected, &actual) {
                        self.errors.push(TypeError::new(
                            format!("Return type mismatch: expected {}, got {}", expected, actual),
                            *span,
                        ));
                    }
                }
                Stmt::If { body, elif_blocks, else_body, .. } => {
                    self.check_return_consistency(body, return_type);
                    for (_, elif_body) in elif_blocks {
                        self.check_return_consistency(elif_body, return_type);
                    }
                    if let Some(eb) = else_body {
                        self.check_return_consistency(eb, return_type);
                    }
                }
                Stmt::While { body, .. } => {
                    self.check_return_consistency(body, return_type);
                }
                Stmt::For { body, .. } => {
                    self.check_return_consistency(body, return_type);
                }
                _ => {}
            }
        }
    }

    /// Check if a type is numeric
    pub(crate) fn is_numeric(&self, t: &Type) -> bool {
        matches!(
            t,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::Int
                | Type::F64
                | Type::F32
                | Type::BigInt
        )
    }

    /// Check if two types are compatible
    pub(crate) fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }

        match (expected, actual) {
            (Type::Infer, _) | (_, Type::Infer) => true,
            // Generic list vs specific list
            (Type::List(e1), Type::List(e2)) => self.is_compatible(e1, e2),
            // Generic auto-conversion and literal narrowing
            (
                Type::I64 | Type::I32 | Type::I16 | Type::I8,
                Type::I64 | Type::I32 | Type::I16 | Type::I8,
            ) => true,
            (Type::F64 | Type::F32, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            (Type::F64 | Type::F32, Type::F64 | Type::F32) => true,

            // BigInt auto-promotion: integer literals/values can be assigned to BigInt
            // This allows: a: BigInt = 0, a: BigInt = 1, return 0 in BigInt fn, etc.
            (Type::BigInt, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            // BigInt can also be compared/assigned to other BigInts
            (Type::BigInt, Type::BigInt) => true,
            // Reverse: BigInt can be implicitly used where int expected (may truncate, but allow)
            (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::BigInt) => true,

            // int (tagged arbitrary precision) is compatible with i64 and vice versa
            (Type::Int, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::Int) => true,
            // int (tagged arbitrary precision) is compatible with BigInt internally
            (Type::Int, Type::BigInt) | (Type::BigInt, Type::Int) => true,

            // Tuples are compatible if their elements are compatible
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return false;
                }
                t1.iter().zip(t2.iter()).all(|(e1, e2)| self.is_compatible(e1, e2))
            }

            // Allow string to byte implicitly to give user standard behavior
            // TODO: this is an oversimplification, ideally would require explicit cast
            (Type::Bytes, Type::Str) => true,

            // Union type compatibility
            // A value of type T is compatible with union T | U
            (Type::Union(variants), _) => {
                variants.iter().any(|variant| self.is_compatible(variant, actual))
            }
            // A union T | U is compatible with another union if all variants are compatible
            (_, Type::Union(variants)) => {
                // For now, be conservative: actual must match at least one variant
                variants.iter().any(|variant| self.is_compatible(expected, variant))
            }

            // Result type compatibility
            (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                self.is_compatible(ok1, ok2) && self.is_compatible(err1, err2)
            }

            // Dict type compatibility
            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
                self.is_compatible(k1, k2) && self.is_compatible(v1, v2)
            }

            // Optional type compatibility
            (Type::Optional(inner1), Type::Optional(inner2)) => self.is_compatible(inner1, inner2),
            // None is compatible with Optional[T]
            (Type::Optional(_), Type::None) => true,

            // GenericApp compatibility (for any remaining generic applications)
            (
                Type::GenericApp { name: n1, type_args: args1 },
                Type::GenericApp { name: n2, type_args: args2 },
            ) => {
                if n1 != n2 || args1.len() != args2.len() {
                    return false;
                }
                args1.iter().zip(args2.iter()).all(|(a1, a2)| self.is_compatible(a1, a2))
            }

            // Class/Instance compatibility
            (Type::Instance(name1), Type::Instance(name2)) => name1 == name2,
            (Type::Class(name1), Type::Class(name2)) => name1 == name2,
            // Instance can match Class of same name
            (Type::Instance(name1), Type::Class(name2)) => name1 == name2,
            (Type::Class(name1), Type::Instance(name2)) => name1 == name2,
            // All classes inherit from Object
            (Type::Object, Type::Instance(_)) => true,
            (Type::Object, Type::Class(_)) => true,
            (Type::Instance(_), Type::Object) => true,
            (Type::Class(_), Type::Object) => true,

            // Method compatibility
            (
                Type::Method {
                    class_name: c1,
                    method_name: m1,
                    params: p1,
                    return_type: r1,
                    is_bound: b1,
                },
                Type::Method {
                    class_name: c2,
                    method_name: m2,
                    params: p2,
                    return_type: r2,
                    is_bound: b2,
                },
            ) => {
                c1 == c2
                    && m1 == m2
                    && b1 == b2
                    && p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(a1, a2)| self.is_compatible(a1, a2))
                    && self.is_compatible(r1, r2)
            }

            // Struct compatibility
            (Type::Struct { name: n1, fields: f1 }, Type::Struct { name: n2, fields: f2 }) => {
                if n1 != n2 || f1.len() != f2.len() {
                    return false;
                }
                f1.iter()
                    .zip(f2.iter())
                    .all(|((name1, t1), (name2, t2))| name1 == name2 && self.is_compatible(t1, t2))
            }

            // TODO: handle user-defined types and interfaces
            _ => false,
        }
    }
}
