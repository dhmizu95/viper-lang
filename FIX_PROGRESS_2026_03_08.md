# Viper Language - Fix Progress Report

**Date:** March 8, 2026  
**Version:** 0.5.0

---

## Summary of Fixes Applied

### 1. Type Checker: Result/Dict/Optional Compatibility ✅

**File:** `src/semantic/type_checker/compatibility.rs`

**Fix:** Added type compatibility rules for:
- `Result[T, E]` types
- `Dict[K, V]` types  
- `Optional[T]` types
- `GenericApp` types
- Class/Instance types
- Method types
- Struct types

**Impact:** Fixed false positive type errors like:
- "Cannot assign {str: int} to {str: int}"
- "Return type mismatch: expected Result[int, str], got Result[int, str]"

### 2. Lexer: Comment Handling ✅

**File:** `src/lexer/scanner.rs`

**Fix:** Properly consume newline after comment lines and reset `start_of_line` flag.

**Before:**
```rust
} else if c == '#' {
    // Skip comment
    while ... { ... }
    continue 'retry;  // Didn't consume newline!
}
```

**After:**
```rust
} else if c == '#' {
    // Skip comment
    while ... { ... }
    // Consume the newline after the comment
    if let Some(&c) = self.chars.peek() {
        if c == '\n' { self.advance(); }
    }
    self.start_of_line = true;
    continue 'retry;
}
```

**Impact:** Fixed "Inconsistent indentation" errors in standard library modules.

### 3. Lexer: Paren Depth Tracking ✅

**File:** `src/lexer/scanner.rs`

**Fix:** Added `paren_depth` field to track nesting inside `()`, `[]`, `{}` and skip indentation handling when inside these delimiters.

**Impact:** Fixed multi-line function signatures like:
```viper
def open(file, mode: str = "r", buffering: int = -1,
         encoding: str = None, errors: str = None,
         newline: str = None, closefd: bool = True):
```

### 4. Missing Builtins ✅

**Files:** `std/builtins_ext.vp`, `std/prelude.vp`, `src/semantic/symbol_table.rs`, `src/semantic/type_checker/exprs.rs`

**Added:**
- `exit(code: int = 0)` - Program exit
- `str_int(s: str, base: int = 10) -> int` - String to int conversion
- `abs_int(n: int) -> int` - Absolute value
- `sqrt_int(n: int) -> int` - Integer square root

**Impact:** Fixed "Undefined function" errors for common builtins.

### 5. Type Checker: String Repetition ✅

**File:** `src/semantic/type_checker/exprs.rs`

**Fix:** Added support for string repetition (`str * int`):
```rust
let is_list_repeat = match (&lt, &rt) {
    ...
    (Type::Str, Type::I64) | (Type::Str, Type::Int) => true,
    (Type::I64, Type::Str) | (Type::Int, Type::Str) => true,
    ...
};
```

**Impact:** Fixed false positive "Arithmetic operators require numeric types" for `"=" * 50`.

---

## Remaining Issues

### 1. Codegen: Binary Operators on Pointer Values ⚠️

**Error:** "Binary operators cannot be applied to pointer values (lists)"

**Example:**
```viper
print("Passed:", passed)  # Comma in print treated as binary op
```

**Root Cause:** The codegen is treating list pointers incorrectly when used with binary operators. This needs investigation in the codegen module.

**Files to Check:**
- `src/codegen/expressions/`
- `src/codegen/runtime/typing.rs`

### 2. Parser: Advanced Syntax ⏸️

**Issues:**
- `raise ... from ...` (exception chaining) - Parser expects Colon, finds Ident
- `@dataclass` decorator - Parser expects Def, finds Class
- `Optional[T]`, `List[T]` typing - Parser expects identifier
- Loop-else syntax - Unexpected token: Indent

**Files to Fix:**
- `src/parser/statements/control_flow.rs`
- `src/parser/statements/definitions.rs`
- `src/parser/expressions.rs`

### 3. Standard Library: io.vp Parser Errors ⏸️

**Error:** "Expected RParen, found None at line 147"

**Root Cause:** Complex function signatures or type annotations in io.vp are not being parsed correctly.

### 4. Standard Library: collections.vp Parser Errors ⏸️

**Error:** "Unexpected token in expression: Tuple"

**Root Cause:** The `tuple` keyword is being encountered in expression context instead of type annotation context.

### 5. Runtime: super() Segfault ⏸️

**Error:** Segmentation fault during `super()` call

**Related:** BUG_JIT_NAME_MAIN_SEGFAULT.md - JIT mode issues with function wrappers.

---

## Test Results Comparison

### Before Fixes
| Category | Passing | Failing |
|----------|---------|---------|
| Core Tests | ~13 | ~20+ |
| Type Checker | Many false positives | - |
| Lexer | Indentation errors | - |
| Builtins | Missing exit, str_int, etc. | - |

### After Fixes (Partial)
| Category | Status |
|----------|--------|
| Type Checker | ✅ Fixed Result/Dict/Optional |
| Lexer | ✅ Fixed comment handling |
| Lexer | ✅ Fixed multi-line signatures |
| Builtins | ✅ Added exit, str_int, abs_int, sqrt_int |
| Type Checker | ✅ Fixed string repetition |
| Codegen | ⚠️ List pointer operators |
| Parser | ⏸️ Advanced syntax |
| Stdlib | ⏸️ io.vp, collections.vp |
| Runtime | ⏸️ super() segfault |

---

## Next Steps

### Immediate (P0)
1. **Fix codegen list pointer operators** - This blocks many tests
2. **Fix parser for raise from** - Exception handling
3. **Fix parser for @dataclass** - Python compatibility

### Short-term (P1)
1. **Fix io.vp parser errors** - Standard library
2. **Fix collections.vp parser errors** - Standard library
3. **Fix super() segfault** - Runtime stability

### Medium-term (P2)
1. **Fix typing module parsing** - Optional[T], List[T]
2. **Fix loop-else syntax** - Control flow
3. **Full stdlib enablement** - csv, datetime, pathlib

---

## Files Modified

1. `src/semantic/type_checker/compatibility.rs` - Type compatibility
2. `src/lexer/scanner.rs` - Comment handling, paren depth
3. `std/builtins_ext.vp` - Added str_int, abs_int, sqrt_int
4. `std/prelude.vp` - Added exit
5. `src/semantic/symbol_table.rs` - Exit builtin registration
6. `src/semantic/type_checker/exprs.rs` - Exit return type, string repetition

---

## Build Status

```
cargo build --release
✅ Compiles with 91 warnings (mostly unused variables)
✅ No errors
```

---

**Estimated Progress:** ~50% of critical issues resolved.
