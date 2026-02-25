# AGENTS.md - Viper Compiler Development Guide

This document provides guidance for agentic coding agents working on the Viper compiler.

## Build Commands

```bash
cargo build              # Debug build
cargo build --release   # Release build with LTO
cargo test              # Run all unit tests
cargo test <test_name>  # Run a specific test (e.g., cargo test test_escape_state_merge)
./run_tests.sh          # Run integration tests (compiles .vp files)
cargo clippy            # Run clippy for warnings
cargo fmt               # Format code
cargo run -- run <file.vp>      # JIT compile and run
cargo run -- build <file.vp> -o <output>  # AOT compile
```

## Project Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library root
├── ast/              # AST node definitions
├── lexer/            # Lexical analysis
├── parser/           # Parsing (recursive descent)
├── semantic/         # Type checking, escape analysis, symbol table
├── codegen/          # LLVM IR generation
│   ├── expressions.rs   # Expression codegen
│   ├── statements.rs    # Statement codegen  
│   ├── control_flow.rs  # If/while/for/return
│   ├── functions.rs     # Function definitions
│   ├── runtime.rs       # Runtime function declarations
│   ├── builder.rs       # IR builder helpers
│   ├── types.rs         # LLVM type mapping
│   ├── variables.rs     # Variable storage (stack/register)
│   ├── state.rs         # Codegen state
│   ├── dce.rs           # Dead code elimination
│   └── generator.rs     # Main CodeGen struct
└── utils/            # Utilities (mangling, spans)
```

## Code Style Guidelines

### Imports
- Use `crate::` for internal imports
- Group: stdlib, external crates, internal modules
- Use `pub use` for re-exports

```rust
use std::collections::HashMap;
use inkwell::values::BasicValueEnum;
use crate::ast::{Expr, Stmt, Type};
use crate::codegen::state::CodeGenState;
```

### Module Organization
- Use `pub mod` for public, `mod` for private
- Add module-level docs with `//!`, file docs with `///`

```rust
//! Statement code generation for Viper

pub mod expressions;
mod helper;
```

### Naming Conventions
- **Types**: PascalCase (`TypeChecker`, `VarInfo`, `EscapeState`)
- **Functions/variables**: snake_case (`generate_expr`, `var_info`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Enums**: PascalCase with CamelCase variants

### Error Handling
- Codegen: `Result<T, String>` for simplicity with LLVM
- Type checker: custom error types with spans

```rust
// Codegen style
pub fn generate_expr(...) -> Result<BasicValueEnum<'ctx>, String> {
    Ok(value)
}

// Type checker style
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: crate::utils::Span,
}
```

### Type Annotations
- Specify types in public interfaces
- Use explicit types for LLVM interop

```rust
pub fn generate_stmt<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
) -> Result<(), String>
```

### Working with LLVM (inkwell)
- Use `.expect()` for operations that should never fail
- Use `?` or `.map_err()` for fallible operations
- Use `unsafe` only when necessary (e.g., GEP operations)

```rust
let elem_ptr = unsafe {
    state.builder.build_in_bounds_gep(elem_type, obj_ptr, &[index_val], "array_elem")
}
.map_err(|e| format!("Failed to build GEP: {:?}", e))?;
```

### Code Generation Patterns
- Allocate variables in entry block for LLVM dominance
- Use stack allocation (alloca) for mutable variables
- Use escape analysis to optimize stack vs heap allocation
- Always verify generated IR with `module.verify()`

### Testing Guidelines
- Add tests in `#[cfg(test)]` modules
- Use descriptive names: `test_<feature>_<behavior>`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape_state_merge() { }
}
```

### Common Patterns

**Option handling**:
```rust
if let Some(value) = get_value() { }
let value = get_value().unwrap_or(default);
```

**Match exhaustive**:
```rust
match expr {
    Variant1 => ...,
    Variant2 => ...,
}
```

## Key Design Decisions

1. **Escape Analysis**: Stack vs heap allocation for performance
2. **Reference Counting**: ARC-style memory management
3. **JIT + AOT**: Both modes supported
4. **LLVM 20**: Uses inkwell with LLVM 20

## External Dependencies

- LLVM 20 (via inkwell)
- clang/GCC for AOT linking
- crates: inkwell, clap, thiserror, which
