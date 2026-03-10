# Remaining Issues Fix Plan

**Date:** March 10, 2026
**Version:** 0.5.0
**Status:** Active Development

---

## Executive Summary

This document outlines the plan to fix remaining issues in the Viper compiler after completing fixes for:
- ✅ f-string literals
- ✅ Bytes literals
- ✅ String concatenation
- ✅ Multiple assignment (tuple unpacking)
- ✅ Float exponent notation (already working)
- ✅ Bitwise operators (already working)
- ✅ Membership operators (already working)

**Related Documents:**
- **Optimization Plan:** See `OPTIMIZATION_PLAN.md` for compiler optimization roadmap
- **Known Issues:** See `KNOWN_ISSUES_FIX_PLAN.md` for recently fixed issues
- **Test Coverage:** See `TEST_COVERAGE_REPORT.md` for current test status

---

## Issue Priority Matrix

| Priority | Issue | Impact | Effort | Status |
|----------|-------|--------|--------|--------|
| P0 | For loops | High - blocks iteration | Medium | In Progress |
| P1 | Import statements | High - blocks modularity | Medium | Pending |
| P1 | Class definitions | High - blocks OOP | High | Pending |
| P2 | Default parameters | Medium - common feature | Low | Pending |
| P2 | Decorators | Medium - common pattern | Medium | Pending |
| P3 | Try/except | Medium - error handling | Medium | Pending |
| P3 | Match statements | Medium - control flow | Medium | Pending |
| P3 | Async/await | Low - advanced feature | High | Pending |

---

## Issue #1: For Loops (P0 - High Priority)

### Problem
For loops only execute the first iteration. While loops work correctly.

**Example:**
```python
def test():
    for i in range(3):
        print(i)  # Only prints 0
    print(999)    # Never prints
```

### Root Cause Analysis
1. **Alloca Placement**: The counter variable alloca is created in the wrong basic block, causing LLVM SSA validation issues
2. **Control Flow**: The step block may not be properly branching back to condition block
3. **Variable Shadowing**: Loop variable may not be properly restored after each iteration

### Files Involved
- `src/codegen/control_flow/loops.rs` - `generate_for()` function (lines 314-500)
- `src/codegen/variables.rs` - `LoopContext` struct

### Fix Plan

#### Phase 1: Debug (1 day)
1. Add LLVM IR dump before JIT execution
2. Compare working while loop IR vs broken for loop IR
3. Identify SSA violations or missing branches

#### Phase 2: Fix (2 days)
1. Move counter alloca to function entry block (proper dominance)
2. Ensure step block always branches to condition block
3. Verify loop variable is properly updated each iteration
4. Fix exit block to properly continue to code after loop

#### Phase 3: Test (1 day)
1. Test simple for loops: `for i in range(n)`
2. Test nested for loops
3. Test for loops with break/continue
4. Test for loops with else clause

### Estimated Time: 4 days

---

## Issue #2: Import Statements (P1 - High Priority)

### Problem
Import statements result in "Undefined variable" error.

**Example:**
```python
import math
print(math.sqrt(16))  # Error: Undefined variable: math
```

### Root Cause Analysis
1. **Module Loading**: Import parsing may work but module loading is not implemented
2. **Symbol Table**: Imported module symbols not added to current scope
3. **Stdlib Path**: Standard library module paths not configured

### Files Involved
- `src/parser/statements/imports.rs` - Import parsing
- `src/codegen/statements/imports.rs` - Import codegen
- `src/semantic/modules.rs` - Module loading
- `std/` - Standard library modules

### Fix Plan

#### Phase 1: Module Loading (2 days)
1. Implement module file discovery in stdlib path
2. Parse and compile imported modules
3. Cache compiled modules to avoid re-compilation

#### Phase 2: Symbol Resolution (2 days)
1. Add imported module symbols to current scope
2. Support `import X`, `from X import Y`, `import X as Z`
3. Handle circular imports gracefully

#### Phase 3: Stdlib Modules (2 days)
1. Fix std/core/math.vp syntax errors
2. Fix std/core/asyncio.vp syntax errors
3. Test common imports: math, random, time, os

### Estimated Time: 6 days

---

## Issue #3: Class Definitions (P1 - High Priority)

### Problem
Class definitions cause LLVM type errors.

**Error:** "Found return instr that returns non-void in Function of void return type!"

### Root Cause Analysis
1. **Method Signature**: Methods without explicit return type default to void
2. **Self Parameter**: `self` parameter type not properly inferred
3. **Instance Type**: Class instance type not properly created

### Files Involved
- `src/codegen/statements/definitions.rs` - Class definition codegen
- `src/codegen/expressions/calls/methods.rs` - Method call codegen
- `src/semantic/types.rs` - Class type handling

### Fix Plan

#### Phase 1: Method Signatures (2 days)
1. Infer return type from method body
2. Default to `None` return type for methods without return
3. Fix `self` parameter type to be instance pointer

#### Phase 2: Instance Creation (2 days)
1. Generate class constructor
2. Allocate instance struct with fields
3. Initialize fields in `__init__`

#### Phase 3: Method Dispatch (2 days)
1. Generate vtable for class methods
2. Support method inheritance
3. Test method calls on instances

### Estimated Time: 6 days

---

## Issue #4: Default Function Parameters (P2 - Medium Priority)

### Problem
Default parameters cause argument mismatch errors.

**Example:**
```python
def greet(name="World"):
    print("Hello", name)

greet()      # Error: argument mismatch
greet("Alice")  # Works
```

### Root Cause Analysis
1. **Function Signature**: Generated function only accepts required parameters
2. **Default Values**: Default values not stored or applied at call site

### Files Involved
- `src/codegen/statements/definitions.rs` - Function definition
- `src/codegen/expressions/calls/core.rs` - Function calls

### Fix Plan

#### Phase 1: Store Defaults (1 day)
1. Parse default parameter values
2. Store defaults in function symbol table entry

#### Phase 2: Generate Wrappers (2 days)
1. Generate overloaded functions for each arity
2. Lower arity functions call full function with defaults

#### Phase 3: Test (1 day)
1. Test single default parameter
2. Test multiple default parameters

### Estimated Time: 4 days

---

## Issue #5: Decorators (P2 - Medium Priority)

### Problem
Decorators result in "Undefined variable" for wrapper function.

### Root Cause Analysis
1. **Nested Functions**: Inner function not properly scoped
2. **Closures**: Decorator needs closure support for captured variables

### Files Involved
- `src/codegen/statements/definitions.rs` - Function definitions
- `src/codegen/variables.rs` - Closure handling

### Fix Plan

#### Phase 1: Nested Functions (2 days)
1. Fix nested function scoping
2. Support closure variable capture

#### Phase 2: Decorator Application (2 days)
1. Parse decorator syntax
2. Apply decorator at function definition time

### Estimated Time: 4 days

---

## Issue #6: Try/Except Blocks (P3 - Medium Priority)

### Problem
Try/except compiles but doesn't execute try block content.

### Files Involved
- `src/codegen/control_flow/exceptions.rs` - Exception codegen

### Fix Plan

#### Phase 1: Basic Try/Except (2 days)
1. Generate proper basic blocks for try/except
2. Ensure try block content executes

#### Phase 2: Exception Types (2 days)
1. Support specific exception types
2. Support multiple except clauses

### Estimated Time: 4 days

---

## Issue #7: Match Statements (P3 - Medium Priority)

### Problem
Match statements compile but don't execute case blocks.

### Files Involved
- `src/codegen/control_flow/match_stmt.rs` - Match codegen

### Fix Plan

#### Phase 1: Basic Match (2 days)
1. Generate comparison chain for cases
2. Proper branching to matching case

#### Phase 2: Advanced Patterns (2 days)
1. Support tuple patterns
2. Support guard conditions

### Estimated Time: 4 days

---

## Issue #8: Async/Await (P3 - Low Priority)

### Problem
Async/await has stdlib parse error in asyncio.vp.

**Error:** "Expected Colon, found Indent at line 59"

### Files Involved
- `std/core/asyncio.vp` - Async stdlib module

### Fix Plan

#### Phase 1: Fix Stdlib (2 days)
1. Fix asyncio.vp syntax errors
2. Fix any other stdlib async modules

#### Phase 2: Async Functions (2 days)
1. Parse `async def` syntax
2. Generate coroutine objects

### Estimated Time: 4 days

---

## Timeline Summary

| Week | Tasks |
|------|-------|
| 1 | Fix for loops (P0) |
| 2 | Fix import statements (P1) |
| 3 | Fix class definitions (P1) |
| 4 | Fix default parameters (P2) |
| 5 | Fix decorators (P2) |
| 6 | Fix try/except (P3) |
| 7 | Fix match statements (P3) |
| 8 | Fix async/await (P3) + cleanup |

**Total Estimated Time: 8 weeks**

---

## Success Criteria

- [ ] For loops iterate correctly through all values
- [ ] Import statements load modules correctly
- [ ] Class definitions compile and methods work
- [ ] Default parameters work for all arities
- [ ] Decorators apply correctly to functions
- [ ] Try/except executes try block and catches exceptions
- [ ] Match statements execute matching case blocks
- [ ] Async/await works with event loop
- [ ] All existing tests still pass (434+)
- [ ] No memory leaks detected

---

*Last Updated: March 10, 2026*
*Version: 0.5.0*
