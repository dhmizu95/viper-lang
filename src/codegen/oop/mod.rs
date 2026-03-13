//! OOP Code Generation Module
//! Handles class instantiation, method calls, attribute access, and inheritance

pub mod classes;

pub use classes::{
    calculate_all_mros, class_exists, generate_attribute_access, generate_class_instantiation,
    generate_class_metadata, generate_field_assignment, generate_user_method_call,
    init_class_registry, with_class_registry, with_class_registry_mut, ClassMetadata,
    ClassRegistry, FieldInfo, MethodInfo,
};
