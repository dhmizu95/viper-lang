pub mod arithmetic;
pub mod bigint;
pub mod comparison;
pub mod core;
pub mod incdec;
pub mod logical;
pub mod membership;
pub mod strings;

pub use core::{generate_binop, generate_unary};
pub use incdec::generate_conditional;
pub use incdec::generate_incdec;
pub use strings::generate_str_concat;
