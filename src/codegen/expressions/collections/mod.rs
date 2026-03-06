//! Collection operations for Viper code generation
//!
//! This module provides functionality for creating and manipulating collections:
//! - Lists: Dynamic arrays with append/get operations
//! - Dicts: Key-value mappings with string keys
//! - Arrays: Fixed-size, stack-allocated arrays
//! - Index/Slice: Access operations for collections

mod lists;
mod dicts;
mod arrays;
mod index;
mod slice;

// Re-export all collection functions
pub use lists::*;
pub use dicts::*;
pub use arrays::*;
pub use index::*;
pub use slice::*;
