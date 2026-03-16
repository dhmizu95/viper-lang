# Lazy Runtime Loading Design for AOT Compilation

## Problem Statement

Currently, all Viper runtime code is compiled into a single static library (`libviper.a`) and linked into every compiled binary, regardless of which features the program actually uses. This results in:
- Larger binary sizes
- Longer linking times
- Increased memory usage at runtime even for simple programs
- Wasted disk space for unused code

## Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Viper Source Code                        │
└─────────────────────┬───────────────────────────────────────┘
                      │ Parsing & Type Checking
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    Codegen (LLVM IR)                        │
│  - Declares all runtime functions (vp_list_*, vp_str_*,   │
│    vp_dict_*, vp_math_*, etc.) as external declarations   │
└─────────────────────┬───────────────────────────────────────┘
                      │ Compilation
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    Linker (GCC/LLD)                        │
│  - Links with libviper.a (contains ALL runtime .o files)  │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              Final Binary (bloated with all runtime)       │
└─────────────────────────────────────────────────────────────┘
```

## Proposed Design

### Approach: Symbol-Based Selective Linking

The key insight is that we can track which runtime functions are actually **declared/used** during codegen, and use this information to selectively link only the necessary runtime components.

#### Step 1: Runtime Module Organization

Group runtime functions into logical modules:

| Module | Functions | Source File |
|--------|-----------|-------------|
| `core` | vp_alloc, vp_free, vp_retain, vp_release, vp_ref_count | runtime.c, memory/ |
| `lists` | vp_list_*, vp_enumerate, vp_zip, vp_list_sum/min/max | data_structures/list.c |
| `dicts` | vp_dict_* | data_structures/dict.c |
| `strings` | vp_str_*, vp_bytes_* | runtime.c (string functions) |
| `tuples` | vp_tuple_* | data_structures/ or runtime.c |
| `math` | vp_math_*, vp_pow, vp_hash_* | runtime.c (math) |
| `print` | vp_print_*, vp_exit | runtime.c (print) |
| `async` | vp_async_*, vp_future_*, vp_fiber_* | async.c, fiber.c, scheduler.c |
| `concurrency` | vp_chan_*, vp_waitgroup_*, vp_thread_* | concurrency/*.c |
| `memoization` | vp_cache_*, vp_lru_cache_* | memoization.c |
| `bigint` | vp_bigint_*, gmp_bridge functions | gmp_bridge.c, tagged_int.c |
| `json` | vp_json_* | json.c |
| `regex` | vp_re_* | re_mod.c |
| `logging` | vp_logging_* | logging.c |

#### Step 2: Codegen Symbol Tracking

Add a symbol tracker in the CodeGen that records which runtime functions are actually used:

```rust
// In codegen/core/mod.rs
pub struct CodeGen {
    // ... existing fields
    pub used_runtime_symbols: HashSet<String>,
}

impl CodeGen {
    pub fn new(...) -> Self {
        // ... existing initialization
        used_runtime_symbols: HashSet::new(),
    }
    
    pub fn track_runtime_symbol(&mut self, symbol: &str) {
        self.used_runtime_symbols.insert(symbol.to_string());
    }
    
    pub fn get_used_modules(&self) -> Vec<&'static str> {
        // Map symbols to modules
    }
}
```

When codegen declares a runtime function (via `module.add_function()`), also record it in the tracker.

#### Step 3: Modular Runtime Libraries

Modify the runtime Makefile to build separate `.a` files:

```
runtime/obj/
├── libviper_core.a     # Core memory management (always needed)
├── libviper_lists.a    # List operations
├── libviper_dicts.a    # Dictionary operations  
├── libviper_strings.a  # String operations
├── libviper_tuples.a   # Tuple operations
├── libviper_math.a     # Math operations
├── libviper_print.a    # Print/formatting
├── libviper_async.a    # Async/await
├── libviper_concurrency.a  # Channels, waitgroups
├── libviper_memoize.a  # Caching
├── libviper_bigint.a   # BigInt (optional)
├── libviper_json.a     # JSON
├── libviper_regex.a    # Regex
└── libviper_logging.a # Logging
```

#### Step 4: Linker Script for Selective Linking

Create a linker script that maps symbols to object files:

```ld
/* runtime/obj/viper.ld */
INPUT(
    libviper_core.a
    libviper_lists.a
    libviper_dicts.a
    libviper_strings.a
    libviper_tuples.a
    libviper_math.a
    libviper_print.a
    libviper_async.a
    libviper_concurrency.a
    libviper_memoize.a
    libviper_bigint.a
    libviper_json.a
    libviper_regex.a
    libviper_logging.a
)
```

#### Step 5: Modified AOT Driver

The AOT driver will:
1. After codegen, collect the set of used runtime symbols
2. Map symbols to required modules
3. Build the linker command with only necessary `.a` files

```rust
// In driver/aot.rs
fn link_with_selective_runtime(
    obj_path: &str,
    bin_path: &str,
    used_modules: &[&str],
    // ... other params
) -> Result<()> {
    let mut args = vec![obj_path.to_string()];
    // ... optimization flags
    
    // Only add required runtime libraries
    for module in used_modules {
        args.push(format!("-lviper_{}", module));
    }
    
    // Always add core
    args.push("-lviper_core".to_string());
    
    // Add system libraries
    args.extend_from_slice(&["-lgmp", "-lm", "-lpthread"]);
    
    // ... execute gcc
}
```

## Implementation Plan

### Phase 1: Symbol Tracking in Codegen

- [ ] Add `used_runtime_symbols: HashSet<String>` to CodeGen struct
- [ ] Modify all `runtime/declare_*` functions to track symbols
- [ ] Add method to extract unique modules from used symbols

### Phase 2: Modular Runtime Build

- [ ] Create new Makefile targets for modular `.a` files
- [ ] Define symbol-to-module mapping
- [ ] Test building individual modules

### Phase 3: Modified AOT Driver

- [ ] Extract used symbols after codegen
- [ ] Map symbols to required modules
- [ ] Build selective linker command

### Phase 4: Testing & Optimization

- [ ] Test with sample programs
- [ ] Measure binary size reduction
- [ ] Verify functionality

## Symbol-to-Module Mapping

```rust
const SYMBOL_TO_MODULE: &[(&str, &str)] = &[
    // Core
    ("vp_alloc", "core"),
    ("vp_free", "core"),
    ("vp_retain", "core"),
    ("vp_release", "core"),
    ("vp_ref_count", "core"),
    
    // Lists
    ("vp_list_create", "lists"),
    ("vp_list_append", "lists"),
    ("vp_list_get", "lists"),
    ("vp_list_set", "lists"),
    ("vp_list_len", "lists"),
    // ... more list functions
    
    // Strings
    ("vp_str_create", "strings"),
    ("vp_str_concat", "strings"),
    ("vp_str_len", "strings"),
    // ... more string functions
    
    // ... etc
];
```

## Expected Benefits

| Program Type | Current Size (est.) | With Lazy Loading |
|-------------|---------------------|-------------------|
| Simple (hello world) | ~2MB | ~200KB |
| List operations | ~2MB | ~500KB |
| Full async program | ~2MB | ~1.5MB |

## Alternative Approaches Considered

1. **Linker GC (--gc-sections)**: Would work but requires whole-program analysis and may not be as effective with static libraries.

2. **Dynamic Loading (dlopen)**: More complex, adds runtime overhead, but provides maximum flexibility.

3. **Compiler-based Dead Code Elimination**: Would require significant changes to LLVM passes.

The proposed approach provides a good balance of implementation complexity and performance improvement.

## Backward Compatibility

- The default behavior should link all modules for simplicity
- Add a flag `--lazy-runtime` to enable selective linking
- Provide a `--link-all-modules` flag to force full linking if needed
