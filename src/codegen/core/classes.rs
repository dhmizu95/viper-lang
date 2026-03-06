//! Class generation methods

use crate::ast::{Stmt, Type};
use inkwell::values::BasicValue;

use crate::codegen::core::context::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    /// Generate code for all class definitions in a module
    pub(crate) fn generate_classes(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        // First pass: collect all class metadata
        for stmt in stmts {
            if let Stmt::Class { name, bases, body, span: _, decorators: _, fields, methods } = stmt {
                let metadata = crate::codegen::oop::generate_class_metadata(
                    name, bases, body, fields, methods
                )?;
                crate::codegen::oop::with_class_registry_mut(|reg| {
                    reg.register_class(metadata);
                });
            }
        }

        // Calculate MRO for all classes
        crate::codegen::oop::with_class_registry_mut(|reg| {
            if let Err(e) = crate::codegen::oop::calculate_all_mros(reg) {
                eprintln!("Warning: Failed to calculate MRO: {}", e);
            }
        });

        // Second pass: generate class code and methods
        for stmt in stmts {
            if let Stmt::Class { name, bases: _, body, span: _, decorators: _, fields, methods } = stmt {
                self.generate_class_def(name, body, fields, methods)?;
            }
        }
        Ok(())
    }

    /// Generate code for a single class definition
    pub(crate) fn generate_class_def(
        &mut self,
        name: &str,
        body: &[Stmt],
        _fields: &[(String, Option<Type>, bool)],
        _methods: &[String],
    ) -> Result<(), String> {
        // Get class metadata from registry
        let metadata = crate::codegen::oop::with_class_registry(|reg| {
            reg.get_class(name).cloned()
        }).ok_or_else(|| format!("Class metadata not found for '{}'", name))?;

        let context = self.context;

        // Create class metadata struct type
        // ViperClass struct layout:
        // - name: i8*
        // - bases: void* (ViperClass**)
        // - base_count: i64
        // - methods: void* (ViperMethod*)
        // - method_count: i64
        // - instance_size: i64
        // - init: function pointer (void*)
        // - dealloc: function pointer (void*)
        let class_struct_type = context.struct_type(&[
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // name
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // bases
            context.i64_type().into(),  // base_count
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // methods
            context.i64_type().into(),  // method_count
            context.i64_type().into(),  // instance_size
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // init
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // dealloc
        ], false);

        // Create class metadata global
        let class_global_name = format!("__viper_class_{}", name);
        let class_global = self.module.add_global(class_struct_type, None, &class_global_name);
        class_global.set_constant(false);
        class_global.set_unnamed_addr(true);

        // Create class name string
        let name_str = self.create_global_string(name);

        // Create initializer values
        let null_ptr = context.ptr_type(inkwell::AddressSpace::default()).const_null();
        let base_count_val = context.i64_type().const_int(0, false);  // Will be updated with inheritance
        let method_count_val = context.i64_type().const_int(metadata.methods.len() as u64, false);
        let instance_size_val = context.i64_type().const_int(metadata.instance_size as u64, false);
        let init_ptr = context.ptr_type(inkwell::AddressSpace::default()).const_null();
        let dealloc_ptr = context.ptr_type(inkwell::AddressSpace::default()).const_null();

        // Create initializer for class struct
        let class_init = class_struct_type.const_named_struct(&[
            name_str.as_basic_value_enum(),  // name
            null_ptr.as_basic_value_enum(),  // bases
            base_count_val.as_basic_value_enum(),  // base_count
            null_ptr.as_basic_value_enum(),  // methods
            method_count_val.as_basic_value_enum(),  // method_count
            instance_size_val.as_basic_value_enum(),  // instance_size
            init_ptr.as_basic_value_enum(),  // init
            dealloc_ptr.as_basic_value_enum(),  // dealloc
        ]);

        class_global.set_initializer(&class_init);

        // Generate method functions
        // Save current class context
        let saved_class = self.current_class.clone();
        self.current_class = Some(name.to_string());

        for stmt in body {
            if let Stmt::Function { name: method_name, params, return_type, body: method_body, decorators, .. } = stmt {
                // Check for staticmethod and classmethod decorators
                let is_static = decorators.iter().any(|d| d.name == "staticmethod");
                let is_class_method = decorators.iter().any(|d| d.name == "classmethod");

                // Generate mangled method name
                let mangled_name = format!("__method_{}_{}", name, method_name);

                // Set flag for classmethod
                let saved_classmethod = self.in_classmethod;
                self.in_classmethod = is_class_method;

                // For static methods, generate as regular function
                // For instance methods, the first param is 'self'
                let empty_nonlocal: Vec<String> = Vec::new();
                if is_static {
                    // Static method - no self parameter
                    self.define_function(&mangled_name, method_name, params, return_type, method_body, &empty_nonlocal)?;
                } else {
                    // Instance method - already has self parameter in AST
                    self.define_function(&mangled_name, method_name, params, return_type, method_body, &empty_nonlocal)?;
                }

                // Restore classmethod flag
                self.in_classmethod = saved_classmethod;
            }
        }

        // Restore previous class context
        self.current_class = saved_class;

        Ok(())
    }
}
