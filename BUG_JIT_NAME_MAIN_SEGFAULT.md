# Bug Report: JIT Segfault with `if __name__ == "__main__"` Pattern

## Status
- **Severity**: High
- **Component**: JIT Runtime / Codegen
- **AOT Mode**: ✅ Working
- **JIT Mode**: ❌ Segfault

## Description

When running Viper code that uses the `if __name__ == "__main__"` pattern in JIT mode, the program segfaults during execution. The same code compiles and runs correctly in AOT mode.

## Reproduction

### Test Case 1: `test_name_main.vp`
```vp
# Test Python-style if __name__ == "__main__" pattern

def main():
    print("This code runs when the script is run directly.")
    print("__name__ =")
    print(__name__)

if __name__ == "__main__":
    main()
```

**AOT Mode:**
```bash
$ ./target/release/viper build test_name_main.vp -o test_name_main
$ ./test_name_main_bin
This code runs when the script is run directly.
__name__ =
__main__
```
✅ Works correctly

**JIT Mode:**
```bash
$ ./target/release/viper run test_name_main.vp
🐍 Viper Compiler 0.4.5 (JIT -O2)
   Running: test_name_main.vp
   Executing via JIT (O2)...
Segmentation fault (core dumped)
```
❌ Segfaults

### Test Case 2: Simple `__name__` access
```vp
print(__name__)
```

**JIT Mode:**
```bash
$ ./target/release/viper run test_name_simple.vp
__main__
✅ Execution complete.
```
✅ Works correctly (no function call)

## Root Cause Analysis

### Hypothesis 1: Entry Point Mismatch

The JIT execution path may not properly handle the `viper_init` → `main` wrapper pattern that was implemented for AOT mode.

**AOT Flow:**
1. `viper_init()` is called first (initializes `__name__`)
2. Wrapper `main()` calls `viper_init()`, then executes module-level code
3. Module-level code calls `__user_main()` via redirected `main()` call

**JIT Flow (suspected issue):**
- JIT may call the user's function directly without going through the wrapper
- `viper_init()` may not be called, leaving `__name__` uninitialized
- Or the function lookup/renaming (`main` → `__user_main`) isn't properly handled

### Hypothesis 2: Function Renaming Issue

The codegen renames user's `main` to `__user_main` and creates a wrapper. The JIT may:
- Look for `main` but find `__user_main` instead
- Call the wrong function entry point
- Not have the wrapper generated at all

### Hypothesis 3: Missing `viper_init` Call

The JIT session may not call `viper_init()` before executing module-level code, resulting in:
- `__name__` being `null`
- String comparison against null pointer → segfault

## Files Involved

### Codegen
- `src/codegen/generator.rs`
  - `generate_main_with_statements()` - Creates wrapper
  - `generate_name_builtin()` - Declares `__name__` global
  - `initialize_name_builtin()` - Initializes `__name__` to `"__main__"`
  - `declare_all_functions()` - Renames `main` to `__user_main`

### JIT Runtime
- `src/repl/session.rs` - JIT execution session
- `src/jit/` (if exists) - JIT compilation logic
- `src/driver/` - Driver code that chooses JIT vs AOT path

### Call Redirection
- `src/codegen/expressions/calls.rs`
  - `generate_user_main_call()` - Redirects `main()` to `__user_main()`

## Suggested Fix Approaches

### Approach 1: Ensure `viper_init` is Called in JIT

Modify the JIT execution path to:
1. Always call `viper_init()` before executing any module-level code
2. Ensure the wrapper `main()` is generated for JIT as well

### Approach 2: Simplify Main Handling

Instead of renaming and wrapping, consider:
1. Keep `main` as-is
2. Generate module-level code that runs before `main`
3. Use a different mechanism to ensure `__name__` is initialized

### Approach 3: JIT-Specific Entry Point

Create a separate entry point generation for JIT that:
1. Doesn't rely on the `main` wrapper pattern
2. Directly initializes `__name__` before JIT execution
3. Handles module-level statements inline

## Debugging Steps

1. **Enable verbose JIT logging:**
   ```bash
   RUST_LOG=debug ./target/release/viper run test_name_main.vp
   ```

2. **Check generated LLVM IR for JIT:**
   - Add IR dump before JIT compilation
   - Compare with AOT IR to identify differences

3. **Use GDB/LLDB:**
   ```bash
   gdb --args ./target/release/viper run test_name_main.vp
   # Run and catch segfault
   # Backtrace to find exact location
   ```

4. **Add debug prints:**
   - In `generate_main_with_statements()`
   - In `initialize_name_builtin()`
   - In JIT session before execution

## Related Changes

This bug was introduced when implementing `__name__` support:
- Added `vp_str_equals` and `vp_str_compare` runtime functions
- Modified string comparison to use content equality instead of pointer equality
- Implemented `main` wrapper pattern for AOT mode

## Workarounds

Until fixed, users can:
1. Use AOT mode: `viper build file.vp && ./file_bin`
2. Avoid `if __name__ == "__main__"` in JIT mode
3. Call `main()` unconditionally for testing

## Test Cases to Add

After fixing, add these to the test suite:
- `tests/02_python_compat/01_name_main/test_jit.vp`
- `tests/02_python_compat/01_name_main/test_aot.vp`
- `tests/02_python_compat/01_name_main/test_import.vp` (imported module `__name__`)

## References

- Python `__name__` semantics: https://docs.python.org/3/library/__main__.html
- Related issue: (link to any existing issues)
- Commit that introduced the feature: (add commit hash when merged)
