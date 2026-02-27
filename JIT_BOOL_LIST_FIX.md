# JIT Bool List Crash - Fix Summary

## Problem
The Viper JIT crashed with a segmentation fault when executing programs using bool lists (e.g., `is_prime = [True] * n`).

## Root Cause
The JIT stubs for bool list operations were incompatible with the ViperList struct layout:

1. **Struct Mismatch**: JIT stubs used `Vec<bool>` directly, but the runtime and codegen expected the `ViperList` struct with specific layout:
   ```c
   struct ViperList {
       int64_t ref_count;    // 8 bytes
       int64_t length;       // 8 bytes
       int64_t capacity;     // 8 bytes
       int64_t elem_type;    // 8 bytes (aligned)
       void* data;           // 8 bytes (points to actual data)
   };  // Total: 40 bytes
   ```

2. **Missing Type-Specific Get**: Codegen called generic `vp_list_get` instead of `vp_list_bool_get` for bool lists.

3. **No Bool List Tracking**: Compiler didn't track which lists were bool lists vs generic i64 lists.

## Solution

### 1. Created Compatible JIT Stubs (`src/jit_stubs/lists_bool.rs`)
```rust
#[repr(C)]
pub struct ViperList {
    pub ref_count: i64,
    pub length: i64,
    pub capacity: i64,
    pub elem_type: i64,
    pub data: *mut c_void,  // Points to Vec<bool>
}

pub extern "C" fn vp_list_bool_repeat_stub(elem: bool, count: i64) -> *mut ViperList {
    let mut vec = Vec::<bool>::new();
    vec.resize(count as usize, elem);
    let data_ptr = Box::into_raw(Box::new(vec)) as *mut c_void;
    
    let list = Box::new(ViperList {
        ref_count: 1,
        length: count,
        capacity: count,
        elem_type: VIPER_LIST_BOOL,
        data: data_ptr,
    });
    
    Box::into_raw(list)
}
```

### 2. Added Bool List Tracking (`src/codegen/state.rs`)
```rust
pub struct CodeGenState<'a, 'ctx> {
    // ... existing fields ...
    pub bool_list_vars: &'a mut HashSet<String>,  // NEW
}

pub fn mark_as_bool_list(&mut self, name: String) {
    self.bool_list_vars.insert(name);
}

pub fn is_bool_list(&self, name: &str) -> bool {
    self.bool_list_vars.contains(name)
}
```

### 3. Updated Codegen to Use Type-Specific Functions (`src/codegen/expressions/collections.rs`)
```rust
// Check if this is a bool list
let is_bool_list = match obj {
    Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
    Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
    // ...
};

// Use bool-specific get for bool lists
if is_bool_list {
    let list_bool_get = state.module.get_function("vp_list_bool_get")?;
    let result = state.ir_builder.build_call(...)?;
    // Convert bool to i64 for compatibility
    return Ok(i64_val.into());
}
```

### 4. Registered JIT Stubs (`src/jit_stubs/registry.rs`)
```rust
// Bool list functions
if let Some(func) = module.get_function("vp_list_bool_create") {
    execution_engine.add_global_mapping(
        &func.as_global_value(), 
        vp_list_bool_create_stub as *const () as usize
    );
}
// ... (repeat for append, get, set, repeat, init_stack, free)
```

## Files Modified

| File | Changes |
|------|---------|
| `src/jit_stubs/lists_bool.rs` | **NEW** - JIT stubs with ViperList compatibility |
| `src/jit_stubs/mod.rs` | Added `lists_bool` module |
| `src/jit_stubs/registry.rs` | Registered bool list stubs |
| `src/codegen/state.rs` | Added `bool_list_vars` tracking |
| `src/codegen/generator.rs` | Added `bool_list_vars` field |
| `src/codegen/statements/core.rs` | Updated function signatures |
| `src/codegen/statements/assignment.rs` | Added bool list detection |
| `src/codegen/expressions/collections.rs` | Type-specific indexing |
| `src/codegen/control_flow/*.rs` | Updated call sites |

## Test Results

### Before Fix
```
Viper JIT: Segmentation fault ❌
```

### After Fix (N=10,000,000)
```
C (gcc -O2):     0.073s
Rust:            0.070s
Go:              0.090s
Viper AOT (-O2): 0.154s  (2.1x slower than C)
Viper JIT (-O2): 0.202s  (2.8x slower than C) ✅ WORKING
```

## Remaining Issues

1. **Performance**: Viper is still 2-3x slower than C/Rust at large scales due to:
   - Function call overhead for list operations
   - No inlining of list access in JIT mode
   - Less aggressive optimization than native compilers

2. **Stack Allocation**: Currently disabled for JIT (works in AOT mode)

## Next Steps

1. **Inline List Operations**: Generate direct LLVM IR for list access instead of function calls
2. **Enable Stack Allocation for JIT**: Fix `vp_list_bool_init_stack_stub` for proper stack semantics
3. **Bit Vector Optimization**: Pack 8 bools per byte for 8x memory savings

## Verification

All implementations correctly identify **664,579 primes** up to 10,000,000.
