# Error Propagation Implementation Status

## Overview
This document describes the implementation of error propagation using the `?` operator and `Result[T, E]` type in the Viper language.

## Implementation Status: PARTIAL ✅

### Completed Features

#### 1. AST Changes (`src/ast/nodes.rs`)
- Added `Unwrap` and `UnwrapOrDefault` variants to `UnaryOp` enum
- The `?` operator is represented as `Expr::UnaryOp { op: UnaryOp::Unwrap, .. }`

#### 2. Lexer (`src/lexer/tokens.rs`, `src/lexer/scanner.rs`)
- `TokenKind::Question` already existed for `?` token
- Lexer correctly tokenizes `?` in expressions

#### 3. Parser (`src/parser/expressions.rs`)
- Added postfix `?` operator parsing in the Pratt parser
- `expr?` is parsed as `Expr::UnaryOp { op: UnaryOp::Unwrap, operand: expr }`
- Properly handles statement boundaries (newlines, dedents)

#### 4. Type System (`src/ast/types.rs`)
- `Type::Result(Box<Type>, Box<Type>)` already existed
- Represents `Result[OkType, ErrType]`

#### 5. Type Inference (`src/semantic/type_checker/infer.rs`)
- `Ok(value)` infers type `Result[value_type, Infer]`
- `Err(error)` infers type `Result[Infer, error_type]`
- `?` operator on `Result[T, E]` returns type `T`

#### 6. Type Checking (`src/semantic/type_checker/exprs.rs`)
- Validates that `?` operator is only used on `Result` types
- Emits error if used on non-Result types
- Checks if function returns Result (simplified check)

#### 7. Codegen for Result Constructors (`src/codegen/expressions/calls.rs`)
- `Ok(value)` creates a struct `{ is_ok: i8, value: i64 }` with `is_ok=1`
- `Err(error)` creates a struct `{ is_ok: i8, value: i64 }` with `is_ok=0`
- Allocates Result on stack

#### 8. Codegen for `?` Operator (`src/codegen/expressions/operators/mod.rs`)
- Generates LLVM IR to check `is_ok` field
- Creates basic blocks for Ok/Err cases
- On Ok: extracts and returns value
- On Err: currently calls `viper_panic` (placeholder for proper propagation)
- Uses phi node to merge values

### Known Limitations

1. **Type Inference**: The error type in `Result[T, E]` is inferred as `Infer` and doesn't unify with explicit annotations yet. For example:
   ```python
   def foo() -> Result[i64, str]:
       return Ok(42)  # Type mismatch: expected Result[i64, str], got Result[i64, _]
   ```

2. **Error Propagation**: The `?` operator currently panics on error instead of properly propagating to the caller. Full implementation needs:
   - Proper return with error value
   - Stack unwinding or exception handling
   - Multiple error type support

3. **Result Representation**: Current implementation uses a simple struct:
   ```llvm
   %Result = type { i8, i64 }
   ```
   This only supports i64 values. Generic support needs:
   - Tagged unions for different types
   - Proper memory layout for complex types
   - ARC integration for reference counting

4. **Helper Methods**: Methods like `.unwrap()`, `.expect()`, `.is_ok()`, `.is_err()`, `.unwrap_err()` are not yet implemented as builtins.

### Files Modified

| File | Changes |
|------|---------|
| `src/ast/nodes.rs` | Added `Unwrap`, `UnwrapOrDefault` to `UnaryOp` |
| `src/parser/expressions.rs` | Added `?` postfix operator parsing |
| `src/semantic/type_checker/infer.rs` | Added Result constructor inference, `?` operator type inference |
| `src/semantic/type_checker/exprs.rs` | Added `?` operator validation |
| `src/codegen/expressions/calls.rs` | Implemented `Ok()` and `Err()` constructors |
| `src/codegen/expressions/operators/mod.rs` | Implemented `generate_unwrap()` for `?` operator |

### Usage Example

```python
# Basic Result usage
def divide(a: i64, b: i64) -> Result[i64, str]:
    if b == 0:
        return Err("Division by zero")
    return Ok(a / b)

# Using ? operator (limited support)
def compute() -> Result[i64, str]:
    x = divide(100, 5)?  # Unwraps or propagates error
    return Ok(x)

# Manual error handling (works now)
def main():
    result = divide(100, 0)
    if result.is_ok():  # TODO: implement is_ok()
        print("Success: " + str(result.unwrap()))  # TODO: implement unwrap()
    else:
        print("Error: " + str(result.unwrap_err()))  # TODO: implement unwrap_err()
```

### Next Steps

1. **Type Unification**: Implement proper type unification to match `Result[T, Infer]` with `Result[T, E]`

2. **Error Propagation**: Implement proper error return instead of panic:
   - Modify function signature to return Result
   - Generate code to construct Err on `?` failure
   - Return early with error value

3. **Helper Methods**: Add builtin methods for Result:
   - `.is_ok()` -> bool
   - `.is_err()` -> bool
   - `.unwrap()` -> T
   - `.unwrap_or_default()` -> T
   - `.expect(msg: str)` -> T
   - `.unwrap_err()` -> E

4. **Generic Result Support**: Support arbitrary Ok and Err types:
   - Proper tagged union representation
   - Memory layout for complex types
   - Integration with ARC

5. **Integration with try/except**: Make `?` operator work with existing exception handling

### Testing

Test files created:
- `tests/test_result.vp` - Comprehensive Result test
- `test_unwrap_simple.vp` - Simple ? operator test

To run tests:
```bash
cargo run -- run test_unwrap_simple.vp
```

## References

- Rust's `?` operator: https://doc.rust-lang.org/std/result/index.html
- Python's exception handling: https://docs.python.org/3/tutorial/errors.html
- LLVM exception handling: https://llvm.org/docs/ExceptionHandling.html
