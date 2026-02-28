# Viper Compiler - Test Fix Plan

## Overview

Fix 3 failing AOT tests caused by missing `fiber_pool.o` in the runtime library build.

**Root Cause:** The Makefile incorrectly compiles `fiber.c`, `fiber_pool.c`, and `scheduler.c` into a single `fiber.o` object file. Since `$<` only uses the first prerequisite, only `fiber.c` is compiled. The `fiber_pool.c` is never compiled, causing undefined references to `vp_fiber_pool_*` functions during linking.

---

## Phase 1: Fix Runtime Makefile ✅ COMPLETED

### 1.1 Add fiber_pool.o Build Target

**File:** `runtime/Makefile`

**Problem:** `fiber_pool.c` is listed in `FIBER_SRCS` but has no dedicated build rule. The `fiber.o` target only compiles the first file (`fiber.c`) because `$<` expands to the first prerequisite.

**Fix 1:** Added `fiber_pool.o` build target after `fiber.o` rule:

```makefile
$(OBJ_DIR)/fiber_pool.o: $(SRC_DIR)/fiber_pool.c
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -pthread -c $< -o $@
```

**Fix 2:** Updated the library target to include `$(OBJ_DIR)/fiber_pool.o`:

Added `$(OBJ_DIR)/fiber_pool.o` between `$(OBJ_DIR)/fiber.o` and `$(OBJ_DIR)/scheduler.o` in the `$(OBJ_DIR)/libviper.a` target.

**Tests fixed:** 
- ✅ `test_concurrency`
- ✅ `test_chan_simple`
- ✅ `test_sync_tasks`

---

## Phase 2: Verification ✅ COMPLETED

```bash
cd runtime && make clean && make
./run_tests.sh
```

**Result:** All 27 tests passing.

---

## Summary

| # | Task | Status |
|---|------|--------|
| 1 | Add `fiber_pool.o` build target to `runtime/Makefile` | ✅ Done |
| 2 | Update library target to include `fiber_pool.o` | ✅ Done |
| 3 | Rebuild runtime library | ✅ Done |
| 4 | Run tests to verify fix | ✅ Done - 27/27 passed |

---

## Notes

- The original plan incorrectly identified the issue as missing JIT stubs
- The actual problem is in the AOT build - `fiber_pool.c` was never compiled
- JIT execution works because it uses Rust stubs in `src/jit_stubs/concurrency.rs`
- AOT builds fail at link time due to missing `vp_fiber_pool_*` symbols
