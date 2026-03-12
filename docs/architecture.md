# Viper Architecture

## Overview
Viper is split into four major layers:

1. Frontend
   - `lexer` tokenizes source into `Token`s.
   - `parser` builds the AST in `ast`.
   - `semantic` performs import loading, symbol registration, type checking, and lightweight analysis such as recursion detection.

2. Middle-end
   - `semantic::constant_folding`
   - `codegen::licm`
   - `codegen::dce`

   These passes currently operate directly on the AST before LLVM IR generation.

3. Backend
   - `codegen` lowers the AST to LLVM IR through Inkwell.
   - `driver::aot` emits object code and links native binaries.
   - `driver::jit` executes through LLVM JIT. The public CLI path runs JIT in an isolated subprocess because LLVM 21 cleanup can crash during teardown.

4. Runtime and standard library
   - `runtime/` contains the C runtime used by generated code.
   - `src/jit_stubs/` provides symbol bindings for JIT execution.
   - `std/` contains Viper standard-library modules.

## Compilation Flow
- A source file is lexed and parsed into `ast::Module`.
- The type checker resolves imports through `module::ModuleLoader` and records import/export data in `module::ModuleRegistry`.
- Frontend analysis identifies recursive functions and emits memoization guidance.
- Optimization passes mutate the AST when enabled.
- Code generation emits LLVM IR.
- AOT uses LLVM tooling plus `gcc` to produce a native binary.
- JIT creates an LLVM execution engine, registers runtime stubs, and invokes `main`.

## Operational Constraints
- AOT depends on `opt`, `gcc`, and the runtime archive in `runtime/obj/libviper.a`.
- JIT has substantial memory overhead because it loads LLVM infrastructure.
- Some multiplication-heavy recursive programs still expose backend/runtime correctness bugs; tests avoid hard-coding those bad results as expected behavior.

## Testing Strategy
- Unit tests cover lexer, parser, semantic utilities, and support code.
- Integration tests execute the built `viper` binary through `CARGO_BIN_EXE_viper`.
- Riskier JIT-facing suites should run with resource limits and single-threaded execution.
