//! Type definitions and LLVM type mapping for Viper code generation

use crate::ast::Type;
use inkwell::context::Context;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;

/// Variable type for codegen
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarType {
    Int,
    Float,
    Pointer,
    Bool,
    Bytes,
}

impl VarType {
    /// Determine VarType from an AST Type
    pub fn from_ast_type(ty: &Type) -> Self {
        match ty {
            Type::F32 | Type::F64 => VarType::Float,
            Type::Bool => VarType::Bool,
            Type::Bytes => VarType::Bytes,
            Type::Str
            | Type::Chan(_)
            | Type::WaitGroup
            | Type::List(_)
            | Type::Dict(_, _)
            | Type::Fn(..)
            | Type::BigInt
            | Type::Int
            | Type::TypeParam { .. }
            | Type::GenericApp { .. }
            | Type::Var(_)
            | Type::Result(_, _) => VarType::Pointer,  // Int uses tagged representation (pointer-sized)
            _ => VarType::Int,
        }
    }
}

/// Type utilities for LLVM code generation
pub struct TypeMapper<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeMapper<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Convert Viper Type to LLVM type
    pub fn llvm_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int => self.context.i64_type().into(),
            Type::F32 | Type::F64 => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),
            Type::Str
            | Type::Bytes
            | Type::Chan(_)
            | Type::WaitGroup
            | Type::List(_)
            | Type::Dict(_, _)
            | Type::Fn(_, _)
            | Type::Optional(_)
            | Type::BigInt => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            Type::Tuple(types) => {
                // Tuples are represented as structs in LLVM
                let field_types: Vec<BasicTypeEnum<'ctx>> =
                    types.iter().map(|t| self.llvm_type(t)).collect();
                if field_types.is_empty() {
                    self.context.i64_type().into()
                } else {
                    self.context.struct_type(&field_types, false).into()
                }
            }
            Type::Array(elem_type, size) => {
                let elem_llvm_type = self.llvm_type(elem_type);
                // array_type is a method on the element type in newer Inkwell
                elem_llvm_type.array_type(*size as u32).into()
            }
            Type::Struct { .. } => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            Type::Future(_) => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            // Result type is represented as a tagged union (pointer-sized)
            Type::Result(_, _) => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            // Type parameters and generic applications are resolved before codegen
            // For unresolved generics, use pointer type as placeholder
            Type::TypeParam { .. } | Type::GenericApp { .. } | Type::Var(_) => {
                self.context.ptr_type(inkwell::AddressSpace::default()).into()
            }
            Type::Infer | Type::Error | Type::None => self.context.i64_type().into(),
            // Union types are represented as tagged unions (pointer-sized)
            Type::Union(_) => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
        }
    }

    /// Get LLVM type for function return
    pub fn llvm_return_type(&self, return_type: &Option<Type>) -> Option<BasicTypeEnum<'ctx>> {
        match return_type {
            Some(Type::I8) | Some(Type::I16) | Some(Type::I32) | Some(Type::I64) | Some(Type::Int) => {
                Some(self.context.i64_type().into())
            }
            Some(Type::F32) | Some(Type::F64) => Some(self.context.f64_type().into()),
            Some(Type::Bool) => Some(self.context.bool_type().into()),
            Some(Type::Str)
            | Some(Type::Chan(_))
            | Some(Type::WaitGroup)
            | Some(Type::List(_))
            | Some(Type::Dict(_, _))
            | Some(Type::Optional(_))
            | Some(Type::BigInt) => {
                Some(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            Some(Type::Tuple(types)) => {
                let field_types: Vec<BasicTypeEnum<'ctx>> =
                    types.iter().map(|t| self.llvm_type(t)).collect();
                if field_types.is_empty() {
                    Some(self.context.i64_type().into())
                } else {
                    Some(self.context.struct_type(&field_types, false).into())
                }
            }
            Some(Type::None) | None => None,
            // Result type uses pointer type (tagged union)
            Some(Type::Result(_, _)) => {
                Some(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            // Generic types and type variables use pointer type
            Some(Type::TypeParam { .. }) | Some(Type::GenericApp { .. }) | Some(Type::Var(_)) => {
                Some(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            // Union types use pointer type
            Some(Type::Union(_)) => Some(self.context.ptr_type(inkwell::AddressSpace::default()).into()),
            _ => Some(self.context.i64_type().into()),
        }
    }
}
