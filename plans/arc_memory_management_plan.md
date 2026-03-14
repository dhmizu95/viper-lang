# Proper ARC Memory Management Implementation Plan

## Problem Statement
The current ARC implementation in Viper is inconsistent and potentially buggy:
1. **Signature Mismatch**: The Viper compiler (codegen) declares `vp_release` with 2 arguments, but the runtime defines it with 1.
2. **Double Refcounting**: Major data structures (`ViperList`, `ViperString`, `ViperDict`, `ViperTuple`) have an internal `ref_count` field at offset 0, but are also prepended by a `ViperHeader` that contains the "real" ARC refcount.
3. **ARC Bypass**: `tagged_int.c` uses manual `malloc`/`free` for `ViperBigInt` and `ViperString` results, bypassing the ARC system.
4. **Collection Leaks**: Collections like `ViperList` don't retain added elements or release them when the list is cleared or destroyed, which leads to leaks for BigInts and other heap-allocated tagged values.

## Proposed Changes

### Phase 1: Correctness Fixes
1. **Fix Codegen Signature**: Update `src/codegen/runtime/memory.rs` to declare `vp_release` with 1 argument (`void* ptr`). Update `src/codegen/state.rs` and other callers to pass only the pointer.
2. **Fix `tagged_int.c` Allocations**:
   - Replace `malloc`/`free` with `vp_arc_alloc`/`vp_arc_release`.
   - Update `alloc_bigint_for_tagged` and `free_bigint_for_tagged` to use the ARC system.
   - Use the unified `ViperString` instead of `MinimalViperString`.

### Phase 2: ARC Header Unification
1. **Remove Redundant Field**: Remove the `int64_t ref_count` field from structs in `runtime/include/viper_types.h`.
2. **Adjust Offsets**: Update `src/codegen/inline_lists.rs` and any other code that relies on field offsets to account for the removed field (all offsets shift by -8).
3. **Macro Update**: Ensure `VP_GET_HEADER` and `VP_GET_OBJECT` are consistent.

### Phase 3: Smart Tagged Management
1. **Implement Tagged ARC Helpers**:
   - Add `vp_tagged_retain(TaggedInt val)` to `tagged_int.c` (retains if BigInt).
   - Add `vp_tagged_release(TaggedInt val)` to `tagged_int.c` (releases if BigInt).
2. **Update Collections**:
   - Update `ViperList` (in `collections.c` and `list.c`) to call `vp_tagged_retain` on elements added and `vp_tagged_release` on elements removed/cleared.
   - Update `ViperDict` and `ViperTuple` similarly.
   - Update destructors to release all elements.

### Phase 4: Verification and Testing
1. **Add Memory Leak Tests**: Create a test case that creates many BigInts, puts them in lists, and then deletes the lists, verifying that memory usage doesn't grow indefinitely.
2. **Check for Regressions**: Run existing benchmark suites (`make bench-safe`) to ensure performance hasn't regressed significantly and correctness is maintained.

## Risk Assessment
- **High Sensitivity**: Changing struct layouts and offsets in codegen is extremely sensitive. A mismatch will lead to crashes or silent data corruption.
- **ABI Compatibility**: Ensure that the C runtime and LLVM codegen agree on the struct layouts.
- **Performance**: Adding retain/release to collection operations might have a small performance impact, but it's necessary for correctness.

## Success Criteria
- No crashes during `make test`.
- No memory leaks for BigInts in collections (validated by manual inspection or valgrind if possible).
- Internally consistent ARC system with a single source of truth for reference counts.
- `tagged_int.c` fully integrated into the ARC system.
