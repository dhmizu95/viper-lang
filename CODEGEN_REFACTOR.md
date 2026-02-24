CodeGen Module Refactoring Plan
 Current State
| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 1616 | Main code generator (monolithic) |
| `builder.rs` | 170 | IR builder helpers |
| `context.rs` | 39 | LLVM context wrapper (unused) |
| `dce.rs` | 326 | Dead code elimination |
 Problem
`mod.rs` handles all domains: types, expressions, statements, runtime functions, control flow, and function calls in a single 1616-line file.
 Proposed Structure
src/codegen/
├── mod.rs           # Main coordinator (thin)
├── builder.rs       # IR builder helpers (keep)
├── dce.rs           # Dead code elimination (keep)
├── types.rs         # Type definitions & LLVM mapping
├── expressions.rs   # Expression code generation
├── statements.rs    # Statement code generation
├── runtime.rs      # Runtime function declarations
├── functions.rs    # Function definition & calls
├── control_flow.rs # If/while/for/return handling
└── variables.rs    # Variable management (VarInfo, VarType)
## Domain Extraction Map
| Module | Extract From | Est. Lines |
|--------|--------------|------------|
| `types.rs` | `VarType`, `llvm_type()`, `llvm_return_type()` | ~80 |
| `variables.rs` | `VarInfo`, `LoopContext`, variable tracking | ~100 |
| `expressions.rs` | `generate_expr()` | ~300 |
| `statements.rs` | `generate_stmt()` | ~250 |
| `runtime.rs` | `declare_runtime_functions()` | ~100 |
| `functions.rs` | Function definition, calls, `functions` HashMap | ~150 |
| `control_flow.rs` | `generate_if()`, `generate_while()`, `generate_for()`, `generate_return()` | ~150 |
## Migration Steps
### Iteration 1 - Foundation
1. Create `types.rs` - extract type-related code
2. Create `variables.rs` - extract `VarInfo`, `LoopContext`
3. Update `mod.rs` imports
### Iteration 2 - Core Domains
4. Create `expressions.rs` - extract expression generation
5. Create `statements.rs` - extract statement generation
6. Create `runtime.rs` - extract runtime function declarations
### Iteration 3 - Further Separation
7. Create `functions.rs` - extract function handling
8. Create `control_flow.rs` - extract control flow constructs
### Iteration 4 - Cleanup
9. Review `context.rs` for removal or keep
10. Refactor `mod.rs` to thin coordinator
## Benefits
- Single responsibility per module
- Easier navigation and debugging
- Testable individual domains
- Reduced merge conflicts
- Parallel development support
## Notes
- Test each iteration before proceeding
- Add integration tests per module
- `CodeGen` struct can delegate to domain modules