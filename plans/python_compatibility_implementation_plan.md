# Python Compatibility Implementation Plan

## Goal

Enable Viper to run the 100 mathematical benchmark problems written in Python syntax with minimal or no modifications. This requires implementing missing lexer, parser, semantic analysis, and codegen features.

---

## Analysis of Missing Features

Based on the benchmark code (files 01-03), the following Python features are used but not supported by Viper:

### Critical (Blocking Parser)

| Feature | Example | Priority |
|---------|---------|----------|
| `import` statements | `import time`, `import math` | P0 |
| Slice assignment with step | `sieve[i*i:limit+1:i] = ...` | P0 |
| `bytearray` type | `bytearray([1]) * (limit + 1)` | P0 |
| f-strings with format specs | `f"{value:,}"`, `f"{value:.3f}"` | P0 |
| List comprehensions with `enumerate` | `[i for i, x in enumerate(sieve) if x]` | P0 |
| `**` power operator | `10**8`, `2**64` | P0 |
| `//` floor division | `n // 2` | P0 |
| `None` default arguments | `def fn(x=None):` | P0 |
| Tuple unpacking | `r, d = 0, n - 1` | P0 |
| `for...else` / `while...else` | `for...else: return False` | P1 |

### High Priority (Runtime Semantics)

| Feature | Example | Priority |
|---------|---------|----------|
| `math.isqrt()` | `int(math.isqrt(limit))` | P0 |
| `time.perf_counter()` | `time.perf_counter()` | P0 |
| `random.seed()`, `random.randint()` | `random.randint(2**60, 2**64-1)` | P0 |
| `pow(base, exp, mod)` | `pow(a, d, n)` | P0 |
| `len()` on bytearray/list | `len(sieve)` | P0 |
| `range(start, stop, step)` | `range(2, limit, i)` | P0 |
| `enumerate()` | `enumerate(sieve)` | P0 |
| `max()`, `min()` | `max(p*p, ...)` | P1 |
| `sum()` | `sum(values)` | P1 |
| `sorted()` | `sorted(factors)` | P1 |
| `int()` conversion | `int(math.isqrt(x))` | P1 |
| `str()` with formatting | `str(value)` | P1 |
| `abs()` | `abs(x - y)` | P1 |
| `math.gcd()` | `math.gcd(a, n)` | P1 |
| `assert` statements | `assert condition` | P1 |
| `global` keyword | `global counter` | P1 |
| `lambda` functions | `lambda x: x * 2` | P1 |
| `continue` in loops | `continue` | P0 |
| `break` in loops | `break` | P0 |

### Medium Priority

| Feature | Example | Priority |
|---------|---------|----------|
| Bitwise operators | `&`, `|`, `^`, `<<`, `>>` | P1 |
| Compound bitwise assignment | `&=`, `|=`, `^=`, `<<=`, `>>=` | P1 |
| `bytes` literals | `b'\x00\x00'` | P1 |
| `list.append()` | `primes.append(n)` | P1 |
| `list.extend()` | `result.extend(items)` | P2 |
| Negative indexing | `primes[-1]` | P1 |
| `if __name__ == "__main__"` | Entry point pattern | P2 |
| Docstrings | `"""docstring"""` | P2 |
| Type hints (ignored) | `def fn(x: int) -> int:` | P2 |

---

## Implementation Plan

### Phase 1: Parser & Lexer (Week 1-2)

#### 1.1 Import System (`import`, `from...import`)

**Files to modify:**
- `src/lexer/tokens.rs` - Already has `Import`, `From`, `As` tokens ✓
- `src/parser/statements/mod.rs` - Add import parsing
- `src/semantic/mod.rs` - Import resolution
- `src/codegen/` - Module loading

**AST Node:**
```rust
pub enum Stmt {
    Import {
        module: String,
        alias: Option<String>,
        span: Span,
    },
    FromImport {
        module: String,
        names: Vec<(String, Option<String>)>, // (name, alias)
        span: Span,
    },
    // ...
}
```

**Standard library modules to implement:**
- `math` - mathematical functions
- `time` - timing functions
- `random` - random number generation

---

#### 1.2 Slice Assignment with Step

**Current status:** Slice expressions are parsed, but slice **assignment** is not supported.

**Files to modify:**
- `src/parser/expressions.rs` - Already parses slices with step ✓
- `src/ast/expressions.rs` - Add `SliceAssign` node
- `src/semantic/` - Type check slice assignment
- `src/codegen/expressions/` - Codegen for slice assignment

**AST Node:**
```rust
pub enum Stmt {
    SliceAssign {
        obj: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
        value: Box<Expr>,
        span: Span,
    },
    // ...
}
```

**LLVM IR pattern:**
```llvm
; sieve[i*i:limit+1:i] = bytearray([0]) * len(...)
; Loop: for (idx = start; idx < end; idx += step) obj[idx] = value
```

---

#### 1.3 `bytearray` Type

**Files to create/modify:**
- `src/semantic/builtins.rs` - Define `bytearray` type
- `src/codegen/expressions/calls.rs` - Handle `bytearray()` constructor
- `runtime/include/viper_bytearray.h` - Runtime support
- `runtime/src/bytearray.c` - Implementation

**Semantics:**
- Mutable sequence of bytes (0-255)
- Supports: indexing, slicing, slice assignment, repetition (`*`), `len()`
- `bytearray([1]) * n` creates n copies

**LLVM Representation:**
```llvm
%bytearray = type { i8*, i64, i64 }  ; { data*, length, capacity }
```

---

#### 1.4 f-strings with Format Specs

**Current status:** Basic f-strings are parsed but format specs (`:,`, `:.3f`) are not supported.

**Files to modify:**
- `src/lexer/scanner.rs` - Parse format specifiers in f-strings
- `src/ast/expressions.rs` - Extend `FString` with format specs
- `src/codegen/expressions/strings.rs` - Format spec codegen

**Format specs to support:**
- `,` - thousands separator
- `.3f` - fixed-point notation
- `d` - decimal integer
- `x` / `X` - hexadecimal
- `b` - binary
- `e` / `E` - scientific notation
- `%` - percentage

---

#### 1.5 List Comprehensions with `enumerate`

**Current status:** Basic list comprehensions are parsed, but `enumerate()` is not implemented.

**Files to modify:**
- `src/semantic/builtins.rs` - Define `enumerate()` builtin
- `src/codegen/expressions/comprehensions.rs` - Handle enumerate in comprehensions

**AST for enumerate:**
```rust
Expr::Call {
    func: Box::new(Expr::Ident("enumerate", span)),
    args: vec![iterable],
    span,
}
```

**LLVM pattern:**
```llvm
; for i, x in enumerate(sieve):
; Equivalent to: for i in range(len(sieve)): x = sieve[i]
```

---

### Phase 2: Operators (Week 2-3)

#### 2.1 Power Operator (`**`)

**Files to modify:**
- `src/lexer/tokens.rs` - `DoubleStar` token exists ✓
- `src/parser/precedence.rs` - Ensure correct precedence
- `src/codegen/expressions/binop.rs` - Codegen for `**`

**Semantics:**
- `int ** int` → arbitrary precision (use BigInt)
- `float ** float` → `pow()` function
- `int ** negative_int` → float result

---

#### 2.2 Floor Division (`//`)

**Files to modify:**
- `src/lexer/tokens.rs` - `DoubleSlash` token exists ✓
- `src/codegen/expressions/binop.rs` - Codegen for `//`

**Semantics:**
- `a // b` = `floor(a / b)`
- For integers: `a / b` rounded toward negative infinity

---

#### 2.3 Bitwise Operators

**Files to modify:**
- `src/lexer/tokens.rs` - Tokens exist: `Ampersand`, `Pipe`, `Caret`, `Tilde`, `LtLt`, `GtGt` ✓
- `src/codegen/expressions/binop.rs` - Codegen

**LLVM mapping:**
- `&` → `and`
- `|` → `or`
- `^` → `xor`
- `~` → `xor -1`
- `<<` → `shl`
- `>>` → `ashr` (arithmetic) or `lshr` (logical)

---

### Phase 3: Built-in Functions (Week 3-4)

#### 3.1 Core Builtins

| Function | Signature | Notes |
|----------|-----------|-------|
| `len()` | `len(seq) -> int` | Works on list, bytearray, str |
| `range()` | `range(stop)`, `range(start, stop)`, `range(start, stop, step)` | Iterator |
| `enumerate()` | `enumerate(iterable, start=0) -> iterator` | Returns (index, value) pairs |
| `pow()` | `pow(base, exp)`, `pow(base, exp, mod)` | 3-arg form is modular exponentiation |
| `abs()` | `abs(x) -> x` | Works on int, float |
| `max()` | `max(a, b)`, `max(iterable)` | |
| `min()` | `min(a, b)`, `min(iterable)` | |
| `sum()` | `sum(iterable, start=0)` | |
| `sorted()` | `sorted(iterable, key=None, reverse=False)` | Returns new list |

**Files to create:**
- `src/semantic/builtins.rs` - Type signatures
- `src/codegen/expressions/builtins.rs` - Codegen

---

### Phase 4: Standard Library Modules (Week 4-6)

#### 4.1 `math` Module

```python
math.isqrt(n)      # Integer square root
math.gcd(a, b)     # Greatest common divisor
math.sqrt(n)       # Floating square root
```

**Files to create:**
- `stdlib/math.vp` - Viper implementation
- Or `runtime/src/math.c` - C implementation for performance

---

#### 4.2 `time` Module

```python
time.perf_counter()  # High-resolution timer
time.time()          # Unix timestamp
```

**Files to create:**
- `runtime/src/time.c` - C implementation using `clock_gettime()`

---

#### 4.3 `random` Module

```python
random.seed(a)           # Initialize RNG
random.randint(a, b)     # Random int in [a, b]
random.getrandbits(k)    # Random k-bit integer
random.choice(seq)       # Random element from sequence
```

**Files to create:**
- `runtime/src/random.c` - C implementation using PCG or xorshift

---

### Phase 5: Statement Support (Week 5-6)

#### 5.1 `for...else` / `while...else`

**Files to modify:**
- `src/parser/statements/for_stmt.rs` - Parse else clause
- `src/codegen/statements/loops.rs` - Codegen with else branch

**Semantics:**
- `else` executes if loop completes without `break`

---

#### 5.2 `global` Keyword

**Files to modify:**
- `src/lexer/tokens.rs` - `Global` token exists ✓
- `src/parser/statements/` - Parse global declarations
- `src/semantic/symbol_table.rs` - Mark symbols as global

---

#### 5.3 `assert` Statements

**Files to modify:**
- `src/lexer/tokens.rs` - `Assert` token exists ✓
- `src/parser/statements/` - Parse assert
- `src/codegen/statements/` - Codegen with optional message

---

### Phase 6: Entry Point & Module System (Week 6-7)

#### 6.1 `if __name__ == "__main__"`

**Files to modify:**
- `src/semantic/` - Recognize `__name__` special variable
- `src/codegen/module.rs` - Generate entry point detection

---

#### 6.2 Module Search Path

**Files to modify:**
- `src/driver/` - Module resolution
- Add `stdlib/` to search path

---

## File Creation Checklist

### Lexer/Parser
- [ ] `src/parser/statements/import_stmt.rs`
- [ ] `src/parser/statements/slice_assign.rs`
- [ ] `src/ast/expressions.rs` - Add `SliceAssign` node
- [ ] `src/ast/statements.rs` - Add `Import`, `FromImport`, `Assert`, `Global`

### Semantic Analysis
- [ ] `src/semantic/builtins.rs` - All builtin functions
- [ ] `src/semantic/modules.rs` - Module resolution
- [ ] `src/stdlib/math.vp`
- [ ] `src/stdlib/time.vp`
- [ ] `src/stdlib/random.vp`

### Codegen
- [ ] `src/codegen/expressions/slice_assign.rs`
- [ ] `src/codegen/expressions/builtins.rs`
- [ ] `src/codegen/expressions/fstring.rs` - Format specs
- [ ] `src/codegen/statements/import.rs`
- [ ] `src/codegen/statements/assert.rs`
- [ ] `src/codegen/statements/loops.rs` - for/while else

### Runtime
- [ ] `runtime/include/viper_bytearray.h`
- [ ] `runtime/src/bytearray.c`
- [ ] `runtime/src/time.c`
- [ ] `runtime/src/random.c`
- [ ] `runtime/src/math.c`

---

## Testing Strategy

### Unit Tests
- Each builtin function
- Each operator
- Each standard library module

### Integration Tests
- Run all 15 Section A benchmarks
- Verify output matches Python reference

### Performance Benchmarks
- Compare JIT vs AOT performance
- Compare against CPython for correctness
- Track performance ratio (Viper/Python)

---

## Milestone: Run All Section A Benchmarks

**Success criteria:**
1. All 15 files parse without errors
2. All 15 files compile (JIT and AOT)
3. All 15 files execute and produce correct output
4. Performance is within 10x of C for integer-heavy benchmarks

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| `bytearray` slice assignment is complex | High | Start with simple cases (no step), add step later |
| f-string format specs are verbose | Medium | Implement `,` and `.Nf` first, add others incrementally |
| Module system requires significant refactoring | High | Implement simple `import` first, defer `from...import` |
| `enumerate()` in comprehensions needs special handling | Medium | Desugar to index-based loop early in pipeline |

---

## Estimated Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. Parser & Lexer | 2 weeks | Import, slice assign, bytearray, f-strings, comprehensions |
| 2. Operators | 1 week | `**`, `//`, bitwise ops |
| 3. Built-in Functions | 1 week | `len`, `range`, `enumerate`, `pow`, etc. |
| 4. Standard Library | 2 weeks | `math`, `time`, `random` modules |
| 5. Statements | 1 week | `for/while...else`, `global`, `assert` |
| 6. Module System | 1 week | `__name__`, module search path |
| **Total** | **8 weeks** | **Full Section A support** |

---

## Notes

- This plan focuses on **Section A (Integer Arithmetic)** benchmarks only
- Sections B-I will require additional work (BigInt, floats, arrays, etc.)
- Prioritize features that unblock multiple benchmarks
- Test incrementally: after each feature, verify which benchmarks now pass
