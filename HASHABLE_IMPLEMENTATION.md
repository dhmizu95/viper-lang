# Hashable Implementation in Viper

## Overview

This document describes the implementation of hashable types and the `hash()` builtin function in the Viper compiler.

## Implementation Summary

### 1. Type System (`src/ast/types.rs`)

Added three methods to the `Type` enum for hashability checking:

- `is_hashable()` - Checks if a type is inherently hashable (int, float, bool, str)
- `is_hashable_tuple()` - Checks if a tuple contains only hashable elements
- `is_fully_hashable()` - Recursively checks if a type is completely hashable

**Hashable built-in types:**
- `i8`, `i16`, `i32`, `i64` - Integer types
- `BigInt` - Arbitrary precision integers
- `f32`, `f64` - Floating point types
- `bool` - Boolean type
- `str` - String type
- `tuple` - If all elements are hashable

**Non-hashable types:**
- `list`, `dict`, `array` - Mutable collections
- Mutable custom objects

### 2. Symbol Table (`src/semantic/symbol_table.rs`)

Added `Hash` to the `BuiltinSignature` enum and registered the `hash()` builtin function that returns `i64`.

### 3. Type Checker (`src/semantic/type_checker.rs`)

Updated dictionary literal checking to validate that all keys are hashable types. Attempts to use non-hashable types as dictionary keys will produce a compile-time error.

Example error:
```
Type errors found:
 - Dictionary keys must be hashable, got [i64] at line 5:8
```

### 4. Runtime Library (`runtime/`)

#### Header (`runtime/include/viper_stdlib.h`)
Added hash function declarations:
```c
int64_t vp_hash_i64(int64_t val);
int64_t vp_hash_f64(double val);
int64_t vp_hash_bool(bool val);
int64_t vp_hash_str(const char* str);
int64_t vp_hash_none(void);
```

#### Implementation (`runtime/src/runtime.c`)
- `vp_hash_i64()` - Uses MurmurHash3 mixer for integer hashing
- `vp_hash_f64()` - Hashes the bit representation of doubles
- `vp_hash_bool()` - Returns 1 for true, 0 for false
- `vp_hash_str()` - Uses FNV-1a hash algorithm for strings
- `vp_hash_none()` - Returns 0 for None values

### 5. Code Generation (`src/codegen/`)

#### Runtime Declaration (`src/codegen/runtime.rs`)
Added `declare_hash_functions()` to register LLVM function declarations for all hash runtime functions.

#### Builtin Call Generation (`src/codegen/expressions/builtins.rs`)
Added `generate_hash_call()` to generate appropriate hash function calls based on argument type:
- Type-specific hash functions for optimal performance
- Special handling for BigInt (converts to string first)
- Proper type checking for bool (i1) vs int (i64)

#### JIT Support (`src/jit_stubs/`)
- Created `hash.rs` with Rust implementations of all hash functions for JIT execution
- Registered hash functions in `registry.rs` for JIT mapping

### 6. Usage Examples

```python
# Basic hash usage
x = hash(42)           # Hash an integer
y = hash(3.14)         # Hash a float
z = hash(True)         # Hash a boolean
s = hash("hello")      # Hash a string

# Hash consistency (same value = same hash)
a = hash("test")
b = hash("test")
print(a == b)          # True

# Dictionary with hashable keys (string keys currently supported)
d = {}
d["key"] = 100
d["another"] = 200

# Tuple keys (future enhancement)
# t = hash((1, 2, 3))  # Requires tuple hash implementation
```

## Current Limitations

### Dictionary Key Support

The current dictionary implementation has the following limitations:

1. **Dict literals** - Type checker validates hashable keys, but runtime only supports string keys
2. **Index assignment** - `d[key] = value` syntax currently only supports i64 indices (for lists)
3. **Integer keys** - Requires `vp_dict_set_i64()` and `vp_dict_get_i64()` implementations
4. **Float keys** - Requires specialized hash-based lookup
5. **Tuple keys** - Requires recursive hashing and equality checking
6. **Generic hash-based dict** - A unified implementation using `vp_hash_*` functions

To use dictionaries in the current implementation, use string keys:
```python
d = {}
d["key"] = 100      # ✓ Works
d["name"] = "Bob"   # ✓ Works
```

### Custom Types

The `__hash__()` and `__eq__()` protocol for custom types is not yet implemented. Future work includes:

1. Adding `__hash__` method support to classes/structs
2. Adding `__eq__` method support for hash collision resolution
3. Validating hash consistency (equal objects must have equal hashes)
4. Documentation on implementing custom hashable types

## Testing

Test file: `tests/hashable_test.vp`

Run tests:
```bash
cargo run -- run tests/hashable_test.vp
```

Expected output:
```
-9148929187392628276
-1558534603216614101
1
-6615550055289275125
True
True
True
All hash tests passed!
✅ Execution complete.
```

## Future Enhancements

1. **Full dict key support** - Implement runtime for all hashable key types
2. **Set type** - Implement set/frozenset using hash-based storage
3. **Custom type hashing** - Support `__hash__()` and `__eq__()` methods
4. **Hash combinators** - Helper functions for combining hashes (useful for tuples)
5. **Salt-based hashing** - Optional salt parameter for hash randomization (security)

## Hash Algorithm Details

### Integer Hash (MurmurHash3 mixer)
```
hash = value
hash ^= hash >> 33
hash *= 0xff51afd7ed558ccd
hash ^= hash >> 33
hash *= 0xc4ceb9fe1a85ec53
hash ^= hash >> 33
```

### String Hash (FNV-1a)
```
hash = 14695981039346656037 (FNV offset basis)
for each byte:
    hash ^= byte
    hash *= 1099511628211 (FNV prime)
```

## Files Modified

- `src/ast/types.rs` - Added hashability checking methods
- `src/semantic/symbol_table.rs` - Added hash builtin signature
- `src/semantic/type_checker.rs` - Added dict key validation
- `src/codegen/runtime.rs` - Added hash function declarations
- `src/codegen/expressions/builtins.rs` - Added hash call generation
- `src/codegen/expressions/calls.rs` - Added hash builtin dispatch
- `runtime/include/viper_stdlib.h` - Added hash function declarations
- `runtime/src/runtime.c` - Implemented hash functions
- `src/jit_stubs/hash.rs` - JIT hash implementations (new file)
- `src/jit_stubs/mod.rs` - Added hash module
- `src/jit_stubs/registry.rs` - Registered hash functions for JIT
- `tests/hashable_test.vp` - Test cases (new file)

## References

- Python hash documentation: https://docs.python.org/3/library/functions.html#hash
- FNV hash: https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
- MurmurHash: https://en.wikipedia.org/wiki/MurmurHash
