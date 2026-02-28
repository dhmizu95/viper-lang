use crate::ast::{Stmt, Type};
use crate::semantic::type_checker::{TypeChecker, TypeError};

impl TypeChecker {
    /// Check that all return statements are consistent with declared return type
    pub(crate) fn check_return_consistency(
        &mut self,
        body: &[Stmt],
        return_type: &Option<Type>,
    ) {
        // If no return type annotation, use Infer to allow any return type
        let expected = return_type.clone().unwrap_or(Type::Infer);

        // Simple check: find all return statements at any nesting level
        // In a real compiler, we would use a visitor pattern for this
        for stmt in body {
            match stmt {
                Stmt::Return { value, span } => {
                    let actual = value
                        .as_ref()
                        .and_then(|e| self.get_expr_type(e))
                        .unwrap_or(Type::None);

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
        matches!(t, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::F64 | Type::BigInt)
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
            (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            (Type::F64 | Type::F32, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            (Type::F64 | Type::F32, Type::F64 | Type::F32) => true,

            // BigInt auto-promotion: integer literals/values can be assigned to BigInt
            // This allows: a: BigInt = 0, a: BigInt = 1, return 0 in BigInt fn, etc.
            (Type::BigInt, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            // BigInt can also be compared/assigned to other BigInts
            (Type::BigInt, Type::BigInt) => true,
            // Reverse: BigInt can be implicitly used where int expected (may truncate, but allow)
            (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::BigInt) => true,

            // int (tagged integer) is compatible with i64 and vice versa
            (Type::Int, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
            (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::Int) => true,

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

            // TODO: handle user-defined types and interfaces
            _ => false,
        }
    }
}
