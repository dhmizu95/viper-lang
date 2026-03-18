# Prime Sieve Optimization Plan

## Current State

| Benchmark | Time | Memory | Status |
|-----------|------|--------|--------|
| Viper AOT (10^9) | ~17s | 1 GB | ✅ Working |
| Viper AOT (10^10) | ~260s | 10 GB | ✅ Working |
| Target (C primesieve) | ~0.4s | ~125 MB | Reference |

## Root Cause Analysis

| Issue | Impact | Severity |
|-------|--------|----------|
| 1 byte per element (not 1 bit) | 8x more memory bandwidth | HIGH |
| Processing even numbers | 2x unnecessary work | MEDIUM |
| List comprehension allocates 50M+ element list | Huge memory + ARC overhead | HIGH |
| Existing SIMD bitvec not exposed | Leaving 10x+ performance on table | HIGH |

## Implementation Status

### ✅ Phase 1: Expose SIMD Bitvec Strided Fill - COMPLETED

**Files Modified:**
- `runtime/src/data_structures/bitvec.c` - Added `vp_bitvec_mark_multiples` wrapper
- `runtime/include/viper_stdlib.h` - Added function declaration
- `src/codegen/runtime/lists.rs` - Added codegen declaration

### ✅ Phase 2: Bitvec Slice Assignment - PARTIALLY COMPLETE

**Files Modified:**
- `src/codegen/statements/assignment.rs` - Added optimization for bitvec slice assignment with False values

**Status:** Partial - code compiles but edge cases need more testing

### ✅ Phase 3: Bitvec Element Assignment - PARTIALLY COMPLETE

**Files Modified:**
- `src/codegen/statements/assignment.rs` - Added `vp_bitvec_set` for bool list element assignment

**Status:** Partial - basic assignment works, some patterns need more testing

### ✅ Phase 4: Bitvec Iteration - PARTIALLY COMPLETE

**Files Modified:**
- `src/codegen/control_flow/loops.rs` - Added `vp_bitvec_get` for iterating over bool lists

**Status:** Partial - basic iteration works, some patterns need more testing

### Pending: Phase 5-6

- Create optimized Viper prime sieve using bitvec (blocked by iteration issues)
- Generator for lazy iteration (future work)

## Test Results

| Test | Result |
|------|--------|
| Bytearray sieve (10^9) | ✅ 17s, works |
| Bytearray sieve (10^10) | ✅ 260s, works |
| Bitvec creation | ✅ Works |
| Bitvec element set | ✅ Works |
| Bitvec slice assign | ✅ Works |
| Bitvec iteration | ⚠️ Partial issues |
