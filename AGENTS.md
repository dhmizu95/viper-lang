# Repository Guidelines

## Project Structure & Module Organization

The compiler lives in [`src/`](D:\Workspace\Projects\viper-lang\src): frontend pieces are under `lexer/`, `parser/`, and `semantic/`, while LLVM lowering is under `codegen/` and execution drivers are under `driver/`. The native runtime is in `runtime/` with C sources in `runtime/src/` and headers in `runtime/include/`. Benchmarks live in `benchmarks/`, standard library sources in `std/`, examples in `examples/`, and test suites in `tests/` with `unit/` and `integration/` coverage. Planning and analysis notes belong in `plans/` and `docs/`.

## Build, Test, and Development Commands

- `make build`: build the Rust compiler.
- `make runtime`: build the C runtime archive used by AOT mode.
- `make test`: run Rust unit and integration tests with `cargo test`.
- `make run`: run the compiler in normal mode.
- `make dev`: run the compiler with `-O0` for easier debugging.
- `make check`: fast Rust compile check without full test execution.
- `make fmt`: format Rust code with `cargo fmt`.
- `make bench` or `make bench-safe`: run Viper benchmarks; prefer safe mode for broader benchmark runs.

For direct CLI work, use `cargo run --bin viper -- run <file.vp>` or `cargo run --bin viper -- build <file.vp>`.

## Coding Style & Naming Conventions

Use `rustfmt` defaults for Rust and keep C runtime formatting consistent with surrounding files, typically 4-space indentation. Prefer `snake_case` for functions, variables, files, and modules; use `CamelCase` for Rust enums/types and runtime structs. Keep changes localized, avoid duplicate codegen paths, and prefer explicit error propagation through the repo’s `Result`/`ViperError` patterns.

## Testing Guidelines

Add unit tests near the subsystem you change and integration tests when behavior crosses parsing, typing, codegen, or runtime boundaries. Test files follow descriptive `test_*` naming in Rust `#[test]` functions. Run `make test` before submitting; for performance-sensitive changes, also run targeted benchmark commands such as `bench-safe-one` or `bench-aot-compare`.

## Commit & Pull Request Guidelines

Recent history favors short imperative subjects, often with conventional prefixes such as `refactor(parser): ...` or `refactor(codegen): ...`. Follow that style: one-line summary, scoped when useful, and keep unrelated refactors separate. Pull requests should include:

- a concise problem statement
- the implementation summary
- tests and benchmark commands run
- performance notes for compiler/runtime changes

Include benchmark deltas or sample CLI output when a change affects code generation or execution speed.
