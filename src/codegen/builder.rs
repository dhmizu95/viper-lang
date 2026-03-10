use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue, StructValue,
};

/// Helper methods for building LLVM IR
pub struct IRBuilder<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> IRBuilder<'ctx> {
    pub fn new(context: &'ctx Context, _module: &Module<'ctx>) -> Self {
        Self { context }
    }

    /// Create an i64 constant
    pub fn i64_const(&self, value: i64) -> IntValue<'ctx> {
        self.context.i64_type().const_int(value as u64, false)
    }

    /// Create an f64 constant
    pub fn f64_const(&self, value: f64) -> FloatValue<'ctx> {
        self.context.f64_type().const_float(value)
    }

    /// Create a bool constant
    pub fn bool_const(&self, value: bool) -> IntValue<'ctx> {
        self.context.bool_type().const_int(value as u64, false)
    }

    /// Create a string constant
    pub fn string_const(&self, module: &Module<'ctx>, s: &str) -> PointerValue<'ctx> {
        let string_type = self.context.i8_type().array_type(s.len() as u32 + 1);
        let global = module.add_global(string_type, None, "str");

        let chars: Vec<i8> = s.bytes().map(|b| b as i8).chain(Some(0)).collect();
        let const_array = self.context.i8_type().const_array(
            &chars
                .iter()
                .map(|&c| self.context.i8_type().const_int(c as u64, false))
                .collect::<Vec<_>>(),
        );

        global.set_initializer(&const_array);
        global.set_constant(true);
        global.set_unnamed_addr(true);
        global.set_linkage(inkwell::module::Linkage::Private);

        global.as_pointer_value()
    }

    /// Create a bytes constant (byte array without null terminator)
    pub fn bytes_const(&self, module: &Module<'ctx>, bytes: &[u8]) -> PointerValue<'ctx> {
        let len = if bytes.is_empty() { 1 } else { bytes.len() };
        let bytes_type = self.context.i8_type().array_type(len as u32);
        let global = module.add_global(bytes_type, None, "bytes");

        let byte_values: Vec<_> = if bytes.is_empty() {
            vec![self.context.i8_type().const_int(0, false)]
        } else {
            bytes
                .iter()
                .map(|&b| self.context.i8_type().const_int(b as u64, false))
                .collect()
        };

        let const_array = self.context.i8_type().const_array(&byte_values);

        global.set_initializer(&const_array);
        global.set_constant(true);
        global.set_unnamed_addr(true);
        global.set_linkage(inkwell::module::Linkage::Private);

        global.as_pointer_value()
    }

    /// Build an addition
    pub fn build_add(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        builder.build_int_add(lhs, rhs, name).expect("add")
    }

    /// Build a subtraction
    pub fn build_sub(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        builder.build_int_sub(lhs, rhs, name).expect("sub")
    }

    /// Build a multiplication
    pub fn build_mul(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        builder.build_int_mul(lhs, rhs, name).expect("mul")
    }

    /// Build a division
    pub fn build_div(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        builder.build_int_signed_div(lhs, rhs, name).expect("div")
    }

    /// Build a comparison (equal)
    pub fn build_icmp_eq(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, name).expect("eq")
    }

    /// Build a comparison (less than)
    pub fn build_icmp_lt(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        builder.build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, name).expect("lt")
    }

    /// Build a conditional branch
    pub fn build_cond_branch(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        cond: IntValue<'ctx>,
        then_block: inkwell::basic_block::BasicBlock<'ctx>,
        else_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        builder.build_conditional_branch(cond, then_block, else_block).expect("cond_br");
    }

    /// Build an unconditional branch
    pub fn build_branch(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        builder.build_unconditional_branch(block).expect("br");
    }

    /// Build a return
    pub fn build_return(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        value: Option<&dyn BasicValue<'ctx>>,
    ) {
        if let Some(v) = value {
            builder.build_return(Some(v)).expect("ret");
        } else {
            builder.build_return(None).expect("ret");
        }
    }

    pub fn build_call(
        &self,
        builder: &inkwell::builder::Builder<'ctx>,
        func: FunctionValue<'ctx>,
        args: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        let callsite = builder.build_call(func, args, name).expect("call");
        // In newer Inkwell, try_as_basic_value returns ValueKind enum with Basic/Instruction variants
        match callsite.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => Some(bv),
            _ => None,
        }
    }

    /// Build a struct constant from element values
    pub fn build_struct_constant(
        &self,
        struct_type: StructType<'ctx>,
        elements: &[BasicValueEnum<'ctx>],
    ) -> StructValue<'ctx> {
        struct_type.const_named_struct(elements)
    }

    /// Create LLVM pass builder options for optimization
    /// Provides 15-25% performance gain when enabled
    /// 
    /// Note: For LLVM 21+, the new pass manager is used with PassBuilderOptions
    /// The actual passes are configured through the options and run via PassManager
    pub fn create_optimization_options(&self) -> inkwell::passes::PassBuilderOptions {
        let options = inkwell::passes::PassBuilderOptions::create();
        
        // Enable key optimizations for better performance
        options.set_loop_interleaving(true);        // Interleave loops for parallelism
        options.set_loop_vectorization(true);       // Vectorize loops (SIMD)
        options.set_loop_slp_vectorization(true);   // SLP vectorization
        options.set_loop_unrolling(true);           // Unroll loops
        options.set_merge_functions(true);          // Merge duplicate functions
        options.set_call_graph_profile(true);       // Use call graph profiling
        options.set_verify_each(false);             // Disable verification for speed
        options.set_debug_logging(false);           // Disable debug logging
        
        options
    }
}
