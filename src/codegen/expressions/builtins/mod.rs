//! Built-in function code generation for Viper
//!
//! This module contains code generation for built-in functions like print, len, str, etc.

pub mod len;
pub mod math;
pub mod print;
pub mod r#struct;
pub mod str;

pub use len::generate_len_call;
pub use math::generate_math_builtin;
pub use print::generate_print_call;
pub use r#struct::{generate_hash_call, generate_struct_pack, generate_struct_unpack};
pub use str::{generate_str_call, generate_type_convert};
