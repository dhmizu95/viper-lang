//! Built-in function code generation for Viper
//!
//! This module contains code generation for built-in functions like print, len, str, etc.

pub mod len;
pub mod math;
pub mod print;
pub mod str;
pub mod r#struct;

pub use len::generate_len_call;
pub use math::{generate_math_builtin, generate_math_constant, generate_math_float_func};
pub use print::{generate_exit_call, generate_print_call};
pub use r#struct::{generate_hash_call, generate_struct_pack, generate_struct_unpack};
pub use str::{generate_str_call, generate_type_convert};
