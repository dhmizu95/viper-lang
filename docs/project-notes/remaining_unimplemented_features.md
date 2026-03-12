# Remaining Unimplemented Or Incomplete Features

## Highest priority correctness gaps
- Multiplication-heavy recursive programs can still produce incorrect results.
- Some memoized recursive programs can still hang.
- These are compiler/runtime bugs, not test-harness bugs.

## Error handling
- The compiler still uses `Result<_, String>` broadly across lexer, parser, module loading, and driver code.
- `src/error.rs` exists, but the full pipeline has not been migrated to structured errors yet.

## CLI / tooling
- `viper test` is still not implemented.
- The CLI now hides it and reports that it is unsupported.

## REPL
- The REPL is still not feature-complete.
- State persistence is based on shadow stores plus source replay, not a true persistent runtime environment.
- Complex values are only tracked loosely.
- Runtime parity with file-based execution is still partial.
- Imported modules now resolve relative to the REPL working path during checking, but imported symbols are not yet fully materialized into persistent REPL runtime state.
- Safety isolation is still weaker than `viper run`.

## Verification
- Targeted bounded suites pass, but there has not been a full bounded sweep of every test/benchmark path in the repository.
