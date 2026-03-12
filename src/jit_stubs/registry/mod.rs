#[macro_use]
mod macros;

pub mod collections;
pub mod concurrency;
pub mod core;
pub mod dispatch;
pub mod io;
pub mod math;
pub mod memoization;
pub mod strings;

pub use collections::register_collection_stubs;
pub use concurrency::register_concurrency_stubs;
pub use core::register_core_stubs;
pub use dispatch::register_stubs;
pub use io::register_io_stubs;
pub use math::register_math_stubs;
pub use memoization::register_memoization_stubs;
pub use strings::register_string_stubs;
