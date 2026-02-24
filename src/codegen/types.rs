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
}

impl VarType {
    /// Determine VarType from an AST Type
    pub fn from_ast_type(ty: &Type) -> Self {
        match ty {
            Type::F32 | Type::F64 => VarType::Float,
            Type::Str | Type::Chan(_) | Type::WaitGroup | Type::List(_) | Type::Dict(_, _) => {
                VarType::Pointer
            }
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
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => self.context.i64_type().into(),
            Type::F32 | Type::F64 => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),
            Type::Str => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            Type::Chan(_) | Type::WaitGroup => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            Type::Array(elem_type, size) => {
                let elem_llvm_type = self.llvm_type(elem_type);
                // array_type is a method on the element type in newer Inkwell
                elem_llvm_type.array_type(*size as u32).into()
            }
            _ => self.context.i64_type().into(),
        }
    }

    /// Get LLVM type for function return
    pub fn llvm_return_type(&self, return_type: &Option<Type>) -> Option<BasicTypeEnum<'ctx>> {
        match return_type {
            Some(Type::I8) | Some(Type::I16) | Some(Type::I32) | Some(Type::I64) => {
                Some(self.context.i64_type().into())
            }
            Some(Type::F32) | Some(Type::F64) => Some(self.context.f64_type().into()),
            Some(Type::Bool) => Some(self.context.bool_type().into()),
            Some(Type::Str) | Some(Type::Chan(_)) | Some(Type::WaitGroup) => Some(
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ),
            Some(Type::None) | None => None,
            _ => Some(self.context.i64_type().into()),
        }
    }
}
