//! Function and method call code generation

mod bigint;
mod builtins_attr;
mod builtins_conv;
mod builtins_inspect;
mod builtins_io;
mod dispatch;
mod functional;
mod lambda;
mod methods;
mod numeric;
mod result;
mod special;
mod super_call;

pub use bigint::*;
pub use builtins_attr::*;
pub use builtins_conv::*;
pub use builtins_inspect::*;
pub use builtins_io::*;
pub use dispatch::*;
pub use functional::*;
pub use lambda::*;
pub use methods::*;
pub use numeric::*;
pub use result::*;
pub use special::*;
pub use super_call::*;
pub use super::generate_expr;
