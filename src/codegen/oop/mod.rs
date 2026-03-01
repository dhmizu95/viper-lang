//! OOP Code Generation Module
//! Handles class instantiation, method calls, attribute access, and inheritance

pub mod classes;

pub use classes::{
    generate_class_instantiation,
    generate_attribute_access,
    generate_user_method_call,
    generate_field_assignment,
    generate_class_metadata,
    init_class_registry,
    class_exists,
    with_class_registry,
    with_class_registry_mut,
    ClassMetadata,
    FieldInfo,
    MethodInfo,
    ClassRegistry,
};
