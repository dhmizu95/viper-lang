//! Utility methods

use crate::ast::Expr;
use inkwell::values::BasicValue;

use crate::codegen::core::context::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    /// Check if an expression can be used as a simple global initializer
    /// Complex types (tuples, lists, dicts, arrays) require runtime allocation
    pub(crate) fn is_simple_initializer_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Int(..)
            | Expr::Float(..)
            | Expr::Bool(..)
            | Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::None(..) => true,
            Expr::UnaryOp { operand, .. } => {
                matches!(operand.as_ref(), Expr::Int(..) | Expr::Float(..))
            }
            _ => false,
        }
    }

    /// Create a global string constant
    pub fn create_global_string(&mut self, s: &str) -> inkwell::values::PointerValue<'ctx> {
        let context = self.context;
        let string_global = self.module.add_global(
            context.i8_type().array_type((s.len() + 1) as u32),
            Some(inkwell::AddressSpace::default()),
            &format!(".str.{}", s.replace(" ", "_").replace("\n", "_n").replace("\"", "_q")),
        );
        string_global.set_constant(true);
        string_global.set_unnamed_addr(true);
        string_global.set_linkage(inkwell::module::Linkage::Private);

        let init_data: Vec<u8> = s.as_bytes().iter().copied().chain(std::iter::once(0)).collect();
        let init_array = context.i8_type().const_array(&init_data.iter()
            .map(|&b| context.i8_type().const_int(b as u64, false))
            .collect::<Vec<_>>());
        string_global.set_initializer(&init_array);

        // GlobalValue is already a pointer, cast it
        string_global.as_basic_value_enum().into_pointer_value()
    }

    /// Verify the generated code
    pub fn verify(&self) -> crate::codegen::Result<()> {
        self.module
            .verify()
            .map_err(|e| crate::codegen::codegen_err(e.to_string()))
    }

    /// Print the generated IR
    #[allow(dead_code)]
    pub fn print_ir(&self) -> String {
        self.module.to_string().to_string()
    }
}
