# Codebase Improvement Implementation Plan

## Summary
Implement the full improvement set in four ordered phases so the repo first becomes correct and internally consistent, then becomes easier to test and maintain. The work should preserve existing language behavior unless a fix is explicitly correcting a bug already identified in the review.

## Implementation Changes
### 1. Correctness and consistency first
- Fix numeric lexing around `.` so number parsing never consumes a dot unless it is confirmed to start a float literal.
- Replace the current "consume then rewind" scanner behavior with explicit lookahead that preserves iterator state.
- Rework module loading to use one loader state machine per compilation, with shared `loaded_modules`, shared `loading_stack`, and propagated import errors.
- Make import resolution consistent between AOT and JIT by introducing one frontend setup path that takes the input file path and constructs lexer, parser, module search path, semantic checker, and recursion analysis identically for both modes.
- Fix recursive memoization warnings so decorators are checked on the specific recursive function being analyzed, not any function in the module.
- Align runtime/build integration:
  - Use the actual runtime archive path produced by the runtime Makefile.
  - Remove the duplicate `runtime/obj/obj` linker search path.
  - Align LLVM version checks/messages with the version in `Cargo.toml`.
  - Ensure prerequisite checks validate the tools the binary really depends on.

### 2. Driver and CLI cleanup
- Introduce a shared compile context/result type used by both `build` and `run` so frontend work is not duplicated.
- Separate "compile", "link", and "execute" concerns in the drivers; JIT should not own frontend setup logic directly.
- Keep the current LLVM 21 MCJIT crash workaround out of the core API surface:
  - Move the forced-process-exit behavior behind a narrow CLI/subprocess boundary.
  - Preserve a library-level execution path that returns normally for internal callers and tests where possible.
- Make CLI metadata accurate and self-consistent:
  - Use package version instead of a stale hardcoded version.
  - Remove unsupported flags from commands that ignore them, or implement the behavior if it is meant to exist.
  - Either implement `viper test` as a real command or remove/hide it until implemented.
- Update user-facing help/info text to match actual capabilities and installation requirements.

### 3. Test architecture overhaul
- Replace shelling out to nested `cargo run` in integration tests with reusable test helpers that invoke library APIs or a narrow subprocess wrapper around the compiled binary.
- Convert "assert `is_ok()`" tests into behavioral assertions on stdout/stderr, exit status, and expected diagnostics.
- Add targeted regression tests for the identified bugs:
  - Number followed by member access or standalone dot token.
  - Relative imports in both `run` and `build`.
  - Circular imports and missing import failures.
  - Multiple recursive functions with mixed decorator coverage.
  - Runtime library path detection and CLI prerequisite messaging.
- Add safe execution helpers for risky tests:
  - Timeouts, memory limits, and single-threaded execution.
  - A dedicated script or helper shared by integration tests/benchmarks rather than ad hoc shelling.
- Keep unit tests fast and deterministic; reserve subprocess-based tests for true integration boundaries.

### 4. Error handling and maintainability
- Replace broad `String` error plumbing in lexer/parser/module loading/driver layers with structured error types carrying source path and span when available.
- Reduce panic/unwrap usage in production code paths where malformed input or environmental failures are possible.
- Keep test-only panics acceptable where they improve failure readability, but remove accidental warnings and dead code.
- Consolidate repo documentation:
  - Move root-level planning/status markdown into `docs/` or `plans/`.
  - Keep the repository root focused on source, build, and top-level project metadata.
- Add a short architecture note describing the compilation pipeline and the division between frontend, codegen, runtime, and CLI.

## Public API / Interface Changes
- Introduce a shared frontend entry point used by both JIT and AOT; it should accept an input path and source text or file path, and return parsed AST plus semantic/module-analysis results.
- Narrow JIT execution into a function that can return `Result<ExecutionOutcome, Error>` without terminating the host process; if a subprocess wrapper is still needed for LLVM cleanup, keep that wrapper at the CLI/test boundary.
- Update CLI command surface so every exposed flag has real behavior and every exposed subcommand is implemented.
- Standardize runtime discovery so both build checks and linker configuration use the same resolved path source.

## Test Plan
- Run `cargo test --no-run` after each phase to catch compile breakage early.
- Run focused suites after each subsystem change:
  - lexer/parser unit tests after scanner changes
  - semantic/module tests after loader/type-checker unification
  - codegen/JIT integration tests after driver refactor
  - CLI/integration tests after command-surface cleanup
- Add explicit regression cases for:
  - `1.foo`, `1..2`, and similar dot-edge lexing
  - relative imports from nested directories
  - circular imports with clear diagnostics
  - uncached recursive function warnings when another recursive function is cached
  - missing runtime archive / wrong LLVM version messaging
- Run bounded integration tests with resource limits and `--test-threads=1` for any subprocess/JIT coverage.
- Final acceptance criteria:
  - `viper run` and `viper build` resolve imports identically
  - no forced hard exit is required in normal library/test code paths
  - CLI help/version/runtime checks match actual behavior
  - integration tests assert output, not just success

## Assumptions
- Scope is the full review, not just the highest-priority compiler bugs.
- Language semantics should remain unchanged except where current behavior is clearly buggy or inconsistent.
- LLVM 21 remains the active toolchain target unless a separate migration plan is requested.
- Runtime Makefile output path is the source of truth for the archive location unless the runtime build layout is intentionally changed as part of implementation.
