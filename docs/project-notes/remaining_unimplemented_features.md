# Remaining Unimplemented Or Incomplete Features

## CLI / tooling
- `viper test` is still not implemented.
- The CLI now hides it and reports that it is unsupported.

## REPL
- The REPL is still not feature-complete.
- State persistence is still based on shadow stores plus source replay, not a true persistent runtime environment.
- Imported modules resolve relative to the REPL working path during checking, but imported symbols are still not fully materialized into persistent REPL runtime state across chunks.
- Runtime parity with file-based execution is still partial.
- Safety isolation is still weaker than `viper run`.

## Frontend architecture
- AOT and JIT share more setup than before, but there is still not one clean compile-context API covering the full frontend pipeline.

## Verification
- Targeted bounded suites and regression checks pass for the fixes completed so far.
- There has still not been a full bounded sweep of every test and benchmark path in the repository.

## Residual risk
- The specific known compiler and runtime bugs previously tracked in this plan are fixed.
- The remaining risk is in unexercised feature combinations and edge cases that do not yet have dedicated regression coverage.
