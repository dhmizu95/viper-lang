//! Class and instance code generation for Viper OOP
//!
//! This module handles:
//! - Class metadata generation
//! - Instance creation (allocation + __init__)
//! - Attribute access (field offsets)
//! - Method calls (vtable lookup)
//! - Inheritance support

use crate::ast::{Expr, Stmt, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::expressions::generate_expr;
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::AddressSpace;
use std::collections::HashMap;
use std::cell::RefCell;

/// Information about a class field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: Type,
    pub offset: usize,
    pub is_class_var: bool,
}

/// Information about a class method
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub mangled_name: String,
    pub is_static: bool,
    pub is_class_method: bool,
    pub is_property: bool,
    pub is_property_setter: bool,  // @name.setter
    pub is_abstract: bool,  // @abstractmethod
    pub property_name: Option<String>,  // For setters, the property name
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

/// Class metadata for code generation
#[derive(Debug, Clone)]
pub struct ClassMetadata {
    pub name: String,
    pub base_classes: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub instance_size: usize,
    pub vtable: HashMap<String, String>, // method_name -> mangled_name
    pub mro: Vec<String>, // Method Resolution Order
}

impl ClassMetadata {
    pub fn new(name: String) -> Self {
        Self {
            name,
            base_classes: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            instance_size: 0,
            vtable: HashMap::new(),
            mro: Vec::new(),
        }
    }

    /// Get instance field by name
    pub fn get_instance_field(&self, field_name: &str) -> Option<&FieldInfo> {
        self.fields.iter()
            .find(|f| f.name == field_name && !f.is_class_var)
    }

    /// Get method info by name
    pub fn get_method(&self, method_name: &str) -> Option<&MethodInfo> {
        self.methods.iter().find(|m| m.name == method_name)
    }

    /// Get method following MRO (includes inherited methods)
    pub fn get_method_mro<'a>(&'a self, method_name: &str, registry: &'a ClassRegistry) -> Option<&'a MethodInfo> {
        // Search through MRO
        for class_name in &self.mro {
            if let Some(class_meta) = registry.classes.get(class_name) {
                if let Some(method) = class_meta.get_method(method_name) {
                    return Some(method);
                }
            }
        }
        None
    }
}

/// Registry of all classes - stored per-generation using thread_local
#[derive(Debug, Default)]
pub struct ClassRegistry {
    classes: HashMap<String, ClassMetadata>,
}

impl ClassRegistry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    pub fn register_class(&mut self, metadata: ClassMetadata) {
        self.classes.insert(metadata.name.clone(), metadata);
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassMetadata> {
        self.classes.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// Get a mutable reference to a class
    pub fn get_class_mut(&mut self, name: &str) -> Option<&mut ClassMetadata> {
        self.classes.get_mut(name)
    }

    /// Find a method by name across all classes (for context manager protocol)
    pub fn find_method(&self, method_name: &str) -> Option<(&ClassMetadata, &MethodInfo)> {
        for (_name, class) in &self.classes {
            if let Some(method) = class.get_method(method_name) {
                return Some((class, method));
            }
        }
        None
    }

    /// Iterate over all classes
    pub fn iter_classes(&self) -> impl Iterator<Item = (&String, &ClassMetadata)> {
        self.classes.iter()
    }
}

/// Calculate MRO using C3 linearization algorithm
/// This implements the same algorithm as Python's MRO
/// 
/// Handles:
/// - Diamond inheritance (A -> B, C -> D where B,C -> A)
/// - Inconsistent hierarchies (where C3 fails)
/// - Provides detailed error messages for debugging
pub fn calculate_mro(
    class_name: &str,
    registry: &ClassRegistry,
) -> crate::codegen::Result<Vec<String>> {
    let class = registry
        .classes
        .get(class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?;

    if class.base_classes.is_empty() {
        return Ok(vec![class_name.to_string()]);
    }

    // Check for duplicate base classes
    let mut seen_bases = std::collections::HashSet::new();
    for base in &class.base_classes {
        if !seen_bases.insert(base) {
            return crate::codegen::codegen_error(format!(
                "Duplicate base class '{}' in class '{}'. Multiple inheritance should not list the same class twice.",
                base, class_name
            ));
        }
    }

    // Check for cycles in inheritance graph
    if let Err(e) = check_inheritance_cycle(class_name, registry, &mut vec![class_name.to_string()]) {
        return Err(e);
    }

    // C3 linearization: merge of parent MROs + parents
    let mut result = Vec::new();
    let mut sequences: Vec<Vec<String>> = Vec::new();

    // Add each parent's MRO
    for base in &class.base_classes {
        if registry.classes.get(base).is_some() {
            let base_mro = calculate_mro(base, registry)?;
            sequences.push(base_mro);
        } else {
            // Base class not found (might be built-in like 'object')
            sequences.push(vec![base.clone()]);
        }
    }

    // Add the list of parents itself
    sequences.push(class.base_classes.clone());

    // Merge sequences using C3 algorithm
    result.push(class_name.to_string());
    merge_sequences(&mut result, &mut sequences, class_name)?;

    Ok(result)
}

/// Check for cycles in the inheritance graph using DFS
fn check_inheritance_cycle(
    class_name: &str,
    registry: &ClassRegistry,
    path: &mut Vec<String>,
) -> crate::codegen::Result<()> {
    let class = match registry.classes.get(class_name) {
        Some(c) => c,
        None => return Ok(()), // External class, can't check
    };

    for base in &class.base_classes {
        // Check if base is already in our path (cycle!)
        if let Some(cycle_start) = path.iter().position(|x| x == base) {
            let mut cycle_path: Vec<_> = path[cycle_start..].to_vec();
            cycle_path.push(base.clone());
            return crate::codegen::codegen_error(format!(
                "Circular inheritance detected: {} -> {}",
                class_name,
                cycle_path.join(" -> ")
            ));
        }

        // Recurse
        path.push(base.clone());
        check_inheritance_cycle(base, registry, path)?;
        path.pop();
    }

    Ok(())
}

/// Merge sequences for C3 linearization
/// Returns detailed error information when the hierarchy is inconsistent
fn merge_sequences(
    result: &mut Vec<String>,
    sequences: &mut [Vec<String>],
    class_name: &str,
) -> crate::codegen::Result<()> {
    let max_iterations = 1000; // Prevent infinite loops
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            return crate::codegen::codegen_error(format!(
                "MRO calculation exceeded maximum iterations for class '{}'. \
                 This may indicate a very complex or pathological inheritance hierarchy.",
                class_name
            ));
        }

        // Check if all sequences are empty
        if sequences.iter().all(|s| s.is_empty()) {
            break;
        }

        // Find a good head (not in any tail)
        let mut found = false;
        for i in 0..sequences.len() {
            if sequences[i].is_empty() {
                continue;
            }

            let head = sequences[i][0].clone();

            // Check if head appears in any tail
            let in_tail = sequences.iter().any(|seq| {
                seq.len() > 1 && seq[1..].iter().any(|x| x == &head)
            });

            if !in_tail {
                // Good head found
                if !result.contains(&head) {
                    result.push(head);
                }
                sequences[i].remove(0);
                found = true;
                break;
            }
        }

        if !found {
            // No good head found - inconsistent hierarchy
            // Provide detailed information about the conflict
            let mut conflict_info = String::new();
            for (i, seq) in sequences.iter().enumerate() {
                if !seq.is_empty() {
                    conflict_info.push_str(&format!(
                        "\n  Sequence {}: [{}]",
                        i,
                        seq.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
                    ));
                    if seq.len() > 5 {
                        conflict_info.push_str("...");
                    }
                }
            }

            return crate::codegen::codegen_error(format!(
                "Inconsistent class hierarchy for class '{}'. \
                 C3 linearization failed due to conflicting inheritance order.{} \n\n\
                 This typically happens when:\n\
                 1. A class inherits from two classes that have incompatible MROs\n\
                 2. Diamond inheritance creates an unresolvable ordering\n\
                 3. A subclass appears before a superclass in the inheritance chain\n\n\
                 Current MRO so far: [{}]\n\
                 Conflicting sequences:{}",
                class_name,
                conflict_info,
                result.join(", "),
                conflict_info
            ));
        }
    }

    Ok(())
}

/// Calculate and set MRO for all classes in the registry
pub fn calculate_all_mros(registry: &mut ClassRegistry) -> crate::codegen::Result<()> {
    let class_names: Vec<String> = registry.classes.keys().cloned().collect();

    for name in class_names {
        let mro = calculate_mro(&name, registry)?;
        if let Some(class) = registry.classes.get_mut(&name) {
            class.mro = mro;
        }
    }

    Ok(())
}

// Thread-local storage for class registry during code generation
thread_local! {
    static CLASS_REGISTRY: RefCell<Option<ClassRegistry>> = const { RefCell::new(None) };
}

/// Initialize the class registry for the current generation
pub fn init_class_registry() {
    CLASS_REGISTRY.with(|r| {
        *r.borrow_mut() = Some(ClassRegistry::new());
    });
}

/// Get a reference to the class registry
pub fn with_class_registry<F, R>(f: F) -> R
where
    F: FnOnce(&ClassRegistry) -> R,
{
    CLASS_REGISTRY.with(|r| {
        let registry = r.borrow();
        let reg = registry.as_ref().expect("Class registry not initialized");
        f(reg)
    })
}

/// Get a mutable reference to the class registry  
pub fn with_class_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut ClassRegistry) -> R,
{
    CLASS_REGISTRY.with(|r| {
        let mut registry = r.borrow_mut();
        let reg = registry.as_mut().expect("Class registry not initialized");
        f(reg)
    })
}

/// Check if a class exists in the registry
pub fn class_exists(name: &str) -> bool {
    with_class_registry(|r| r.contains(name))
}

/// Generate class metadata from a class definition statement
pub fn generate_class_metadata(
    name: &str,
    bases: &[Expr],
    body: &[Stmt],
    decorators: &[crate::ast::Decorator],
    fields: &[(String, Option<Type>, bool)],
    _methods: &[String],
) -> crate::codegen::Result<ClassMetadata> {
    let mut metadata = ClassMetadata::new(name.to_string());

    // Check for @dataclass decorator on class
    let is_dataclass = decorators.iter().any(|d| d.name == "dataclass");

    // Process base classes
    for base in bases {
        if let Expr::Ident(base_name, _) = base {
            metadata.base_classes.push(base_name.clone());
        }
    }

    // Process fields
    let mut current_offset = 0usize;

    // Add fields from the definition
    for (field_name, type_ann, is_class_var) in fields {
        let ty = type_ann.clone().unwrap_or(Type::Infer);
        let field_size = get_type_size(&ty);

        metadata.fields.push(FieldInfo {
            name: field_name.clone(),
            ty,
            offset: current_offset,
            is_class_var: *is_class_var,
        });

        if !is_class_var {
            current_offset += field_size;
        }
    }

    // For @dataclass, also collect annotated fields from class body
    if is_dataclass {
        for stmt in body {
            if let Stmt::Declare { name: field_name, type_ann: Some(ty), .. } = stmt {
                // Add field if not already present
                if !metadata.fields.iter().any(|f| f.name == *field_name) {
                    let field_size = get_type_size(ty);
                    metadata.fields.push(FieldInfo {
                        name: field_name.clone(),
                        ty: ty.clone(),
                        offset: current_offset,
                        is_class_var: false,
                    });
                    current_offset += field_size;
                }
            }
        }
    }

    // Scan body for instance fields (self.x assignments in __init__)
    for stmt in body {
        if let Stmt::Function { name: method_name, body: method_body, .. } = stmt {
            if method_name == "__init__" {
                // Scan for self.field assignments
                for stmt in method_body {
                    scan_self_assignments(stmt, &mut metadata, &mut current_offset);
                }
            }
        }
    }

    metadata.instance_size = current_offset.max(8); // Minimum 8 bytes

    // For @dataclass, generate __init__, __repr__, __eq__ methods
    if is_dataclass {
        let instance_fields: Vec<(String, Type)> = metadata.fields.iter()
            .filter(|f| !f.is_class_var)
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect();
        
        // Check if methods already exist
        let has_init = metadata.methods.iter().any(|m| m.name == "__init__");
        let has_repr = metadata.methods.iter().any(|m| m.name == "__repr__");
        let has_eq = metadata.methods.iter().any(|m| m.name == "__eq__");
        
        // Generate missing methods
        if !has_init {
            generate_dataclass_init_method(&mut metadata.methods, name, &instance_fields);
        }
        if !has_repr {
            generate_dataclass_repr_method(&mut metadata.methods, name, &instance_fields);
        }
        if !has_eq {
            generate_dataclass_eq_method(&mut metadata.methods, name, &instance_fields);
        }
    }

    // Process methods
    for stmt in body {
        if let Stmt::Function {
            name: method_name,
            params,
            return_type,
            decorators,
            ..
        } = stmt {
            let is_static = decorators.iter().any(|d| d.name == "staticmethod");
            let is_class_method = decorators.iter().any(|d| d.name == "classmethod");
            let is_abstract = decorators.iter().any(|d| d.name == "abstractmethod");

            // Check for property getter (@property) or setter (@name.setter)
            let is_property = decorators.iter().any(|d| d.name == "property");
            let mut is_property_setter = false;
            let mut property_name: Option<String> = None;
            
            // Check for @name.setter pattern
            for dec in decorators {
                if dec.name.contains('.') {
                    let parts: Vec<&str> = dec.name.split('.').collect();
                    if parts.len() == 2 && parts[1] == "setter" {
                        is_property_setter = true;
                        property_name = Some(parts[0].to_string());
                        break;
                    }
                }
            }

            let param_types: Vec<(String, Type)> = params.iter()
                .map(|p| (p.name.clone(), p.type_ann.clone().unwrap_or(Type::Infer)))
                .collect();

            let mangled_name = format!("__method_{}_{}", name, method_name);

            metadata.methods.push(MethodInfo {
                name: method_name.clone(),
                mangled_name: mangled_name.clone(),
                is_static,
                is_class_method,
                is_property,
                is_property_setter,
                is_abstract,
                property_name: property_name.clone(),
                params: param_types,
                return_type: return_type.clone().unwrap_or(Type::None),
            });

            // For property setters, also add to vtable under the property name
            if is_property_setter {
                if let Some(prop_name) = property_name {
                    // Store setter with a special key
                    metadata.vtable.insert(format!("{}.setter", prop_name), mangled_name.clone());
                }
            } else {
                metadata.vtable.insert(method_name.clone(), mangled_name);
            }
        }
    }

    Ok(metadata)
}

/// Scan statements for self.* assignments to find instance fields
fn scan_self_assignments(stmt: &Stmt, metadata: &mut ClassMetadata, offset: &mut usize) {
    match stmt {
        Stmt::Assign { target, value, .. } => {
            if let Expr::Attribute { obj, attr, .. } = target.as_ref() {
                if let Expr::Ident(obj_name, _) = obj.as_ref() {
                    if obj_name == "self" {
                        // Check if field already exists
                        if !metadata.fields.iter().any(|f| f.name == *attr) {
                            // Infer field type from the assigned value
                            let field_type = infer_type_from_expr(value);
                            metadata.fields.push(FieldInfo {
                                name: attr.clone(),
                                ty: field_type,
                                offset: *offset,
                                is_class_var: false,
                            });
                            *offset += 8; // All fields are pointer-sized (8 bytes)
                        }
                    }
                }
            }
        }
        Stmt::If { body, elif_blocks, else_body, .. } => {
            for stmt in body { scan_self_assignments(stmt, metadata, offset); }
            for (_, elif_body) in elif_blocks {
                for stmt in elif_body { scan_self_assignments(stmt, metadata, offset); }
            }
            if let Some(else_body) = else_body {
                for stmt in else_body { scan_self_assignments(stmt, metadata, offset); }
            }
        }
        Stmt::While { body, .. } => {
            for stmt in body { scan_self_assignments(stmt, metadata, offset); }
        }
        Stmt::For { body, .. } => {
            for stmt in body { scan_self_assignments(stmt, metadata, offset); }
        }
        _ => {}
    }
}

/// Infer a field type from an expression
fn infer_type_from_expr(expr: &crate::ast::Expr) -> crate::ast::Type {
    use crate::ast::Expr;
    match expr {
        Expr::Int(..) | Expr::BigInt(..) => crate::ast::Type::Int,
        Expr::Float(..) => crate::ast::Type::F64,
        Expr::Bool(..) => crate::ast::Type::Bool,
        Expr::Str(..) => crate::ast::Type::Str,
        Expr::Bytes(..) => crate::ast::Type::Bytes,
        Expr::List { .. } | Expr::ListComprehension { .. } => crate::ast::Type::List(Box::new(crate::ast::Type::Infer)),
        Expr::Dict { .. } => crate::ast::Type::Dict(Box::new(crate::ast::Type::Infer), Box::new(crate::ast::Type::Infer)),
        Expr::Tuple { elements, .. } => crate::ast::Type::Tuple(elements.iter().map(infer_type_from_expr).collect()),
        Expr::None(..) => crate::ast::Type::None,
        Expr::Call { func, .. } => {
            // Check if calling a known type constructor
            if let Expr::Ident(name, _) = func.as_ref() {
                match name.as_str() {
                    "int" => crate::ast::Type::Int,
                    "float" => crate::ast::Type::F64,
                    "str" => crate::ast::Type::Str,
                    "bool" => crate::ast::Type::Bool,
                    "list" => crate::ast::Type::List(Box::new(crate::ast::Type::Infer)),
                    "dict" => crate::ast::Type::Dict(Box::new(crate::ast::Type::Infer), Box::new(crate::ast::Type::Infer)),
                    _ => crate::ast::Type::Infer,
                }
            } else {
                crate::ast::Type::Infer
            }
        }
        // For other expressions, default to pointer type
        _ => crate::ast::Type::Str,
    }
}

/// Get the size of a type in bytes
fn get_type_size(ty: &Type) -> usize {
    match ty {
        Type::I8 | Type::Bool => 1,
        Type::I16 => 2,
        Type::I32 => 4,
        Type::I64 | Type::Int | Type::F64 | Type::Str | Type::List(_) | Type::Dict(_, _) 
        | Type::Class(_) | Type::Instance(_) | Type::BigInt => 8,
        Type::F32 => 4,
        Type::Tuple(types) => types.iter().map(get_type_size).sum(),
        _ => 8,
    }
}

/// Generate code for class instantiation: ClassName(args...)
pub fn generate_class_instantiation<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    class_name: &str,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let metadata = with_class_registry(|r| {
        r.get_class(class_name).cloned()
    }).ok_or_else(|| format!("Class '{}' not found", class_name))?;

    // Check if class has abstract methods
    let abstract_methods: Vec<&MethodInfo> = metadata.methods.iter()
        .filter(|m| m.is_abstract)
        .collect();
    
    if !abstract_methods.is_empty() {
        // Check if this is a direct instantiation (not from subclass)
        // For now, emit a runtime error for abstract class instantiation
        let abstract_names: Vec<&str> = abstract_methods.iter()
            .map(|m| m.name.as_str())
            .collect();
        
        eprintln!("   warning: Class '{}' has abstract methods: {}", 
                  class_name, abstract_names.join(", "));
        // In a full implementation, this would generate a runtime check
        // For now, we just warn and allow instantiation (for subclassing)
    }

    // Allocate memory for the instance
    let instance_ptr = allocate_instance(state, &metadata)?;

    // Call __init__ if it exists
    if let Some(init_method) = metadata.get_method("__init__") {
        let mangled_name = &init_method.mangled_name;
        
        if let Some(func_val) = state.functions.get(mangled_name).copied() {
            // Build argument list: self + user args
            let mut arg_values: Vec<_> = args.iter()
                .map(|a| generate_expr(state, a)
                    .map(|v| inkwell::values::BasicMetadataValueEnum::from(v)))
                .collect::<Result<_, _>>()?;
            
            // Insert self as first argument
            arg_values.insert(0, instance_ptr.into());

            state.ir_builder.build_call(
                state.builder,
                func_val,
                &arg_values,
                "init_call",
            );
        }
    }

    Ok(instance_ptr.into())
}

/// Allocate memory for a class instance
fn allocate_instance<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    metadata: &ClassMetadata,
) -> crate::codegen::Result<PointerValue<'ctx>> {
    let i64_type = state.context.i64_type();
    
    // Get malloc function
    let malloc_func = state.module.get_function("malloc")
        .or_else(|| state.module.get_function("vp_alloc"))
        .ok_or_else(|| "malloc/vp_alloc not found".to_string())?;
    
    // Allocate memory for instance
    let size_val = i64_type.const_int(metadata.instance_size as u64, false);
    
    let result = state.ir_builder.build_call(
        state.builder,
        malloc_func,
        &[size_val.into()],
        "alloc_instance",
    );
    
    let ptr = result.ok_or_else(|| "Failed to allocate instance".to_string())?
        .into_pointer_value();
    
    Ok(ptr)
}

/// Generate code for attribute access: obj.field
pub fn generate_attribute_access<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    attr_name: &str,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Generate object expression to get instance pointer
    let obj_val = generate_expr(state, obj)?;

    if !obj_val.is_pointer_value() {
        return crate::codegen::codegen_error("Attribute access on non-object type".to_string());
    }

    let obj_ptr = obj_val.into_pointer_value();

    // Try to determine class type from object expression
    let class_name = infer_class_type(state, obj);

    if let Some(class_name) = class_name {
        if let Some(metadata) = with_class_registry(|r| r.get_class(&class_name).cloned()) {
            // Check if it's a field access
            if let Some(field) = metadata.get_instance_field(attr_name) {
                return generate_field_access(state, obj_ptr, field);
            }

            // Check if it's a property
            if let Some(method) = metadata.get_method(attr_name) {
                if method.is_property {
                    return generate_property_getter(state, obj_ptr, method);
                }
            }
        }
    }

    crate::codegen::codegen_error(format!("Unknown attribute '{}' on object", attr_name))
}

/// Generate field access with offset calculation
fn generate_field_access<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj_ptr: PointerValue<'ctx>,
    field: &FieldInfo,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let i64_type = state.context.i64_type();
    let i8_ptr_type = state.context.ptr_type(AddressSpace::default());

    // Calculate field address: obj_ptr + offset
    let offset_val = i64_type.const_int(field.offset as u64, false);

    // Cast obj_ptr to i8* for byte arithmetic
    let obj_i8 = state.builder.build_bit_cast(obj_ptr, i8_ptr_type, "obj_i8")
        .map_err(|e| format!("Failed to cast object: {:?}", e))?
        .into_pointer_value();

    // Get field pointer
    let field_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            state.context.i8_type(),
            obj_i8,
            &[offset_val],
            &format!("field_{}_ptr", field.name),
        )
    }.map_err(|e| format!("Failed to calculate field offset: {:?}", e))?;

    // All fields are stored as i64 (pointer-sized)
    // Reference types store the pointer value as i64
    // Float types need special handling
    let value = if field.ty == Type::F64 {
        // Float field - load as f64 then bitcast to i64 for uniform handling
        let field_f64_ptr = state.builder.build_bit_cast(
            field_ptr,
            state.context.ptr_type(AddressSpace::default()),
            "field_f64_ptr"
        ).map_err(|e| format!("Failed to cast field ptr: {:?}", e))?
        .into_pointer_value();
        
        let f64_val = state.builder.build_load(state.context.f64_type(), field_f64_ptr, &format!("field_{}", field.name))
            .map_err(|e| format!("Failed to load field: {:?}", e))?;
        
        // Bitcast f64 to i64 for uniform return
        state.builder.build_float_to_signed_int(f64_val.into_float_value(), i64_type, "f64_to_i64")
            .map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
            .into()
    } else if field.ty == Type::Bool {
        // Bool field - load as i8 then zero-extend to i64
        let field_bool_ptr = state.builder.build_bit_cast(
            field_ptr,
            state.context.ptr_type(AddressSpace::default()),
            "field_bool_ptr"
        ).map_err(|e| format!("Failed to cast field ptr: {:?}", e))?
        .into_pointer_value();
        
        let bool_val = state.builder.build_load(state.context.i8_type(), field_bool_ptr, &format!("field_{}", field.name))
            .map_err(|e| format!("Failed to load field: {:?}", e))?;
        
        // Zero-extend i8 to i64
        state.builder.build_int_z_extend(bool_val.into_int_value(), i64_type, "bool_to_i64")
            .map_err(|e| format!("Failed to convert bool to i64: {:?}", e))?
            .into()
    } else {
        // Default: load as i64 (handles pointers, ints, etc.)
        state.builder.build_load(i64_type, field_ptr, &format!("field_{}", field.name))
            .map_err(|e| format!("Failed to load field: {:?}", e))?
    };

    Ok(value)
}

/// Generate property getter call
fn generate_property_getter<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj_ptr: PointerValue<'ctx>,
    method: &MethodInfo,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if let Some(func_val) = state.functions.get(&method.mangled_name).copied() {
        let result = state.ir_builder.build_call(
            state.builder,
            func_val,
            &[obj_ptr.into()],
            "property_get",
        );
        
        Ok(result.unwrap_or(state.context.i64_type().const_int(0, false).into()))
    } else {
        crate::codegen::codegen_error(format!("Property getter '{}' not found", method.name))
    }
}

/// Generate code for method call: obj.method(args...)
pub fn generate_user_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    method_name: &str,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Generate object expression
    let obj_val = generate_expr(state, obj)?;

    if !obj_val.is_pointer_value() {
        return crate::codegen::codegen_error("Method call on non-object type".to_string());
    }

    let obj_ptr = obj_val.into_pointer_value();

    // Try to determine class type
    let class_name = infer_class_type(state, obj);

    if let Some(class_name) = class_name {
        if let Some(metadata) = with_class_registry(|r| r.get_class(&class_name).cloned()) {
            if let Some(method) = metadata.get_method(method_name) {
                if method.is_static {
                    return generate_static_method_call(state, method, args);
                }
                if method.is_class_method {
                    return generate_class_method_call(state, &class_name, method, args);
                }
                return generate_instance_method_call(state, obj_ptr, method, args);
            }
        }
    }

    crate::codegen::codegen_error(format!("Method '{}' not found on object", method_name))
}

/// Generate static method call
fn generate_static_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    method: &MethodInfo,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if let Some(func_val) = state.functions.get(&method.mangled_name).copied() {
        let arg_values: Vec<_> = args.iter()
            .map(|a| generate_expr(state, a)
                .map(|v| inkwell::values::BasicMetadataValueEnum::from(v)))
            .collect::<Result<_, _>>()?;
        
        let result = state.ir_builder.build_call(
            state.builder,
            func_val,
            &arg_values,
            "static_method_call",
        );
        
        Ok(result.unwrap_or(state.context.i64_type().const_int(0, false).into()))
    } else {
        crate::codegen::codegen_error(format!("Static method '{}' not found", method.name))
    }
}

/// Generate instance method call
fn generate_instance_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj_ptr: PointerValue<'ctx>,
    method: &MethodInfo,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if let Some(func_val) = state.functions.get(&method.mangled_name).copied() {
        // Build argument list: self + user args
        let mut arg_values: Vec<_> = args.iter()
            .map(|a| generate_expr(state, a)
                .map(|v| inkwell::values::BasicMetadataValueEnum::from(v)))
            .collect::<Result<_, _>>()?;
        
        // Insert self as first argument
        arg_values.insert(0, obj_ptr.into());
        
        let result = state.ir_builder.build_call(
            state.builder,
            func_val,
            &arg_values,
            &format!("call_{}", method.name),
        );
        
        Ok(result.unwrap_or(state.context.i64_type().const_int(0, false).into()))
    } else {
        crate::codegen::codegen_error(format!("Method '{}' not found", method.name))
    }
}

/// Generate classmethod call - passes class pointer instead of self
fn generate_class_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    class_name: &str,
    method: &MethodInfo,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if let Some(func_val) = state.functions.get(&method.mangled_name).copied() {
        // Build argument list: class metadata pointer + user args
        let mut arg_values: Vec<_> = args.iter()
            .map(|a| generate_expr(state, a)
                .map(|v| inkwell::values::BasicMetadataValueEnum::from(v)))
            .collect::<Result<_, _>>()?;

        // For classmethods, pass a pointer to the class metadata global
        // This allows the method to access class-level information
        let class_global_name = format!("__viper_class_{}", class_name);
        let class_ptr = if let Some(global) = state.module.get_global(&class_global_name) {
            global.as_pointer_value()
        } else {
            // Fallback to null pointer if class global not found
            state.context.ptr_type(inkwell::AddressSpace::default()).const_null()
        };
        
        arg_values.insert(0, class_ptr.into());

        let result = state.ir_builder.build_call(
            state.builder,
            func_val,
            &arg_values,
            &format!("classmethod_{}", method.name),
        );

        Ok(result.unwrap_or(state.context.i64_type().const_int(0, false).into()))
    } else {
        crate::codegen::codegen_error(format!("Class method '{}' not found", method.name))
    }
}

/// Infer the class type from an expression
fn infer_class_type<'ctx>(
    state: &CodeGenState<'_, 'ctx>,
    expr: &Expr,
) -> Option<String> {
    match expr {
        Expr::Ident(name, _) => {
            // Look up the variable's class name from the codegen state
            state.variables.get(name).and_then(|var| var.class_name.clone())
        }
        Expr::Call { func, .. } => {
            // If this is ClassName(), return ClassName
            if let Expr::Ident(class_name, _) = func.as_ref() {
                Some(class_name.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generate code for field assignment: obj.field = value
pub fn generate_field_assignment<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    field_name: &str,
    value: &Expr,
) -> crate::codegen::Result<()> {
    let obj_val = generate_expr(state, obj)?;

    if !obj_val.is_pointer_value() {
        return crate::codegen::codegen_error("Field assignment on non-object type".to_string());
    }

    let obj_ptr = obj_val.into_pointer_value();
    let value_val = generate_expr(state, value)?;

    // Try to determine class type
    let class_name = infer_class_type(state, obj);

    if let Some(class_name) = class_name {
        if let Some(metadata) = with_class_registry(|r| r.get_class(&class_name).cloned()) {
            // First check for property setter
            let setter_key = format!("{}.setter", field_name);
            if let Some(setter_mangled) = metadata.vtable.get(&setter_key) {
                // Call the property setter method
                if let Some(func_val) = state.functions.get(setter_mangled).copied() {
                    state.ir_builder.build_call(
                        state.builder,
                        func_val,
                        &[obj_ptr.into(), value_val.into()],
                        "property_set",
                    );
                    return Ok(());
                }
            }
            
            // Fall back to direct field assignment
            if let Some(field) = metadata.get_instance_field(field_name) {
                return store_field(state, obj_ptr, field, value_val);
            }
        }
    }

    crate::codegen::codegen_error(format!("Field '{}' not found on object", field_name))
}

/// Store value to a field - all fields stored as i64 (pointer-sized)
fn store_field<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj_ptr: PointerValue<'ctx>,
    field: &FieldInfo,
    value: BasicValueEnum<'ctx>,
) -> crate::codegen::Result<()> {
    let i64_type = state.context.i64_type();
    let i8_ptr_type = state.context.ptr_type(AddressSpace::default());

    // Calculate field address
    let offset_val = i64_type.const_int(field.offset as u64, false);

    // Cast obj_ptr to i8* for byte arithmetic
    let obj_i8 = state.builder.build_bit_cast(obj_ptr, i8_ptr_type, "obj_i8")
        .map_err(|e| format!("Failed to cast object: {:?}", e))?
        .into_pointer_value();

    // Get field pointer
    let field_ptr = unsafe {
        state.builder.build_in_bounds_gep(
            state.context.i8_type(),
            obj_i8,
            &[offset_val],
            &format!("field_{}_ptr", field.name),
        )
    }.map_err(|e| format!("Failed to calculate field offset: {:?}", e))?;

    // Convert value to i64 for storage if needed
    let store_value = if field.ty == Type::F64 && value.is_float_value() {
        // Float field - convert f64 to i64 bits for storage
        state.builder.build_float_to_signed_int(value.into_float_value(), i64_type, "f64_to_i64")
            .map_err(|e| format!("Failed to convert f64 to i64: {:?}", e))?
            .into()
    } else if field.ty == Type::Bool && value.is_int_value() && value.get_type().into_int_type().get_bit_width() == 1 {
        // Bool field - zero-extend i1 to i64 for storage
        state.builder.build_int_z_extend(value.into_int_value(), i64_type, "bool_to_i64")
            .map_err(|e| format!("Failed to convert bool to i64: {:?}", e))?
            .into()
    } else {
        // Already i64 or pointer (stored as i64)
        value
    };

    // Store as i64
    state.builder.build_store(field_ptr, store_value)
        .map_err(|e| format!("Failed to store field: {:?}", e))?;

    Ok(())
}

/// Generate __init__ MethodInfo for dataclass
fn generate_dataclass_init_method(
    methods: &mut Vec<MethodInfo>,
    class_name: &str,
    fields: &[(String, Type)],
) {
    // Build params: self + all fields
    let mut params = vec![("self".to_string(), Type::Instance(class_name.to_string()))];
    for (field_name, field_type) in fields {
        params.push((field_name.clone(), field_type.clone()));
    }
    
    methods.push(MethodInfo {
        name: "__init__".to_string(),
        mangled_name: format!("__method_{}___init__", class_name),
        is_static: false,
        is_class_method: false,
        is_property: false,
        is_property_setter: false,
        is_abstract: false,
        property_name: None,
        params,
        return_type: Type::None,
    });
}

/// Generate __repr__ MethodInfo for dataclass
fn generate_dataclass_repr_method(
    methods: &mut Vec<MethodInfo>,
    class_name: &str,
    _fields: &[(String, Type)],
) {
    methods.push(MethodInfo {
        name: "__repr__".to_string(),
        mangled_name: format!("__method_{}___repr__", class_name),
        is_static: false,
        is_class_method: false,
        is_property: false,
        is_property_setter: false,
        is_abstract: false,
        property_name: None,
        params: vec![("self".to_string(), Type::Instance(class_name.to_string()))],
        return_type: Type::Str,
    });
}

/// Generate __eq__ MethodInfo for dataclass
fn generate_dataclass_eq_method(
    methods: &mut Vec<MethodInfo>,
    class_name: &str,
    _fields: &[(String, Type)],
) {
    methods.push(MethodInfo {
        name: "__eq__".to_string(),
        mangled_name: format!("__method_{}___eq__", class_name),
        is_static: false,
        is_class_method: false,
        is_property: false,
        is_property_setter: false,
        is_abstract: false,
        property_name: None,
        params: vec![
            ("self".to_string(), Type::Instance(class_name.to_string())),
            ("other".to_string(), Type::Object),
        ],
        return_type: Type::Bool,
    });
}
