# Viper Language Comprehensive Test Plan

This document outlines a comprehensive testing strategy for the Viper programming language, covering all language features using Python-compatible syntax.

## Test Organization

Tests are organized into the following categories:
1. **Lexer Tests** - Token recognition and lexical analysis
2. **Parser Tests** - Syntax parsing and AST generation
3. **Semantic Analysis Tests** - Type checking and symbol resolution
4. **Code Generation Tests** - LLVM IR generation and execution
5. **Standard Library Tests** - Built-in modules functionality
6. **Integration Tests** - End-to-end feature combinations

---

## 1. Lexer Tests (tests/lexer/)

### 1.1 Literals
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_int_literals.vp | Integer literals | 42, -17, 0xFF, 0b1010, 0o755 |
| test_float_literals.vp | Float literals | 3.14, 2.5e10, 1.0e-5, 3.14_159 |
| test_string_literals.vp | String literals | "hello", 'world', """multiline""" |
| test_fstring_literals.vp | f-string literals | f"value={x}", f"{expr!r}" |
| test_bytes_literals.vp | Bytes literals | b"bytes", b"\x00\xff" |
| test_bigint_literals.vp | BigInt literals | 123n, large integers |
| test_bool_literals.vp | Boolean literals | True, False |
| test_none_literal.vp | None literal | None |

### 1.2 Operators
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_arithmetic_ops.vp | Arithmetic operators | +, -, *, /, //, %, ** |
| test_comparison_ops.vp | Comparison operators | ==, !=, <, <=, >, >= |
| test_logical_ops.vp | Logical operators | and, or, not |
| test_bitwise_ops.vp | Bitwise operators | &, |, ^, ~, <<, >> |
| test_identity_ops.vp | Identity operators | is, is not |
| test_membership_ops.vp | Membership operators | in, not in |
| test_augmented_assign.vp | Augmented assignment | +=, -=, *=, /=, //=, %=, **=, &=, |=, ^=, <<=, >>= |
| test_walrus_operator.vp | Assignment expression | (x := expr) |
| test_null_coalesce.vp | Null coalescing | x ?? default |
| test_ternary.vp | Conditional expression | x if cond else y |

### 1.3 Delimiters and Keywords
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_delimiters.vp | Delimiters | (), [], {}, ,, :, ;, ., -> |
| test_keywords.vp | All keywords | def, class, if, for, while, etc. |
| test_indentation.vp | Python-style indentation | Indent/Dedent tokens |

---

## 2. Parser Tests (tests/parser/)

### 2.1 Expressions
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_binary_ops.vp | Binary operations | a + b * c, operator precedence |
| test_unary_ops.vp | Unary operations | -x, +y, ~z, not flag |
| test_function_calls.vp | Function calls | func(), f(a, b), f(*args, **kwargs) |
| test_indexing.vp | Index access | arr[0], matrix[i][j] |
| test_slicing.vp | Slice access | lst[:], lst[1:], lst[:5], lst[1:10:2] |
| test_attribute_access.vp | Attribute access | obj.attr, a.b.c |
| test_list_literals.vp | List literals | [], [1, 2, 3], [x for x in range(10)] |
| test_tuple_literals.vp | Tuple literals | (), (1,), (a, b, c) |
| test_dict_literals.vp | Dictionary literals | {}, {"key": "value"}, {k: v for k, v in items} |
| test_array_literals.vp | Array literals | [0; 10], [1, 2, 3] |
| test_lambda.vp | Lambda expressions | lambda x: x + 1, lambda a, b: a * b |
| test_comprehensions.vp | List comprehensions | [x*2 for x in range(10) if x % 2 == 0] |
| test_await.vp | Await expressions | await future, await asyncio.sleep(1) |
| test_super.vp | Super calls | super().__init__() |

### 2.2 Statements
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_assign.vp | Assignment | x = 1, a, b = b, a, x, *rest = seq |
| test_declare.vp | Variable declaration | x: int = 0, name: str |
| test_const.vp | Constant declaration | const PI = 3.14159 |
| test_global_nonlocal.vp | Global/Nonlocal | global x, nonlocal count |
| test_if.vp | If statements | if cond: ... elif ... else: ... |
| test_while.vp | While loops | while cond: ..., while True: ... break |
| test_for.vp | For loops | for i in range(10): ..., async for |
| test_match.vp | Match statements | match x: case 1: ... case _: ... |
| test_try_except.vp | Exception handling | try: ... except E: ... finally: ... |
| test_with.vp | Context managers | with open(f) as f: ..., async with |
| test_raise.vp | Raise statements | raise Exception(), raise E from cause |
| test_assert.vp | Assert statements | assert condition, assert cond, "message" |
| test_delete.vp | Delete statements | del x, del lst[0], del obj.attr |
| test_yield.vp | Yield statements | yield value, yield from gen |
| test_return.vp | Return statements | return, return value, return a, b |
| test_import.vp | Import statements | import module, from mod import name as alias |
| test_extern.vp | External functions | extern "C" def func(args) -> ret |

### 2.3 Definitions
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_function_def.vp | Function definitions | def func(a, b=0, *args, **kwargs) -> T: ... |
| test_async_def.vp | Async functions | async def fetch(): ... |
| test_decorators.vp | Decorators | @decorator, @decorator(args), @a.b.c |
| test_class_def.vp | Class definitions | class Name(Base): ..., class Generic[T]: ... |
| test_struct_def.vp | Struct definitions | struct Point { x: f64, y: f64 } |
| test_type_alias.vp | Type aliases | type Point = (f64, f64) |

### 2.4 Concurrency
| Test File | Feature | Python Syntax |
|-----------|---------|---------------|
| test_sync.vp | Sync blocks | sync: ... |
| test_task.vp | Task spawning | task func() |
| test_channels.vp | Channels | chan(10), send(c, v), recv(c) |
| test_waitgroup.vp | WaitGroup | wg = WaitGroup(), add(wg, n), done(wg), wait(wg) |
| test_select.vp | Select statements | select: case recv(c): ... case default: ... |

---

## 3. Semantic Analysis Tests (tests/semantic/)

### 3.1 Type Checking
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_type_inference.vp | Type inference | Hindley-Milner style inference |
| test_type_annotations.vp | Type annotations | Explicit type checking |
| test_generics.vp | Generic types | def identity[T](x: T) -> T: ... |
| test_union_types.vp | Union types | int | str, type narrowing |
| test_optional_types.vp | Optional types | T?, null checks |
| test_result_types.vp | Result types | Result[Ok, Err], error propagation |
| test_type_bounds.vp | Type bounds | T: Hashable + Comparable |

### 3.2 Symbol Resolution
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_scope_resolution.vp | Scope resolution | Local, enclosing, global, built-in |
| test_closure_detection.vp | Closure detection | Free variable capture |
| test_shadowing.vp | Variable shadowing | Inner scope shadows outer |

### 3.3 Error Detection
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_type_errors.vp | Type errors | Mismatched types |
| test_undefined_vars.vp | Undefined variables | Use before definition |
| test_duplicate_defs.vp | Duplicate definitions | Same name in scope |

---

## 4. Code Generation Tests (tests/codegen/)

### 4.1 Basic Operations
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_arithmetic_codegen.vp | Arithmetic | Integer and float operations |
| test_comparison_codegen.vp | Comparisons | All comparison operators |
| test_logical_codegen.vp | Logical | Short-circuit evaluation |

### 4.2 Data Structures
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_list_codegen.vp | Lists | Dynamic array operations |
| test_tuple_codegen.vp | Tuples | Fixed-size collections |
| test_dict_codegen.vp | Dicts | Hash map operations |
| test_string_codegen.vp | Strings | String operations and methods |

### 4.3 Control Flow
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_branches_codegen.vp | Branches | if/elif/else codegen |
| test_loops_codegen.vp | Loops | while/for loop codegen |
| test_match_codegen.vp | Match | Pattern matching codegen |

### 4.4 Functions and Closures
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_functions_codegen.vp | Functions | Call/return codegen |
| test_closures_codegen.vp | Closures | Environment capture |
| test_recursion_codegen.vp | Recursion | Recursive calls |

### 4.5 OOP
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_classes_codegen.vp | Classes | Class layout and methods |
| test_inheritance_codegen.vp | Inheritance | Base class support |
| test_virtual_calls_codegen.vp | Virtual calls | Dynamic dispatch |

### 4.6 Concurrency
| Test File | Feature | Description |
|-----------|---------|-------------|
| test_async_codegen.vp | Async/await | Coroutine codegen |
| test_channels_codegen.vp | Channels | Channel operations |
| test_select_codegen.vp | Select | Select statement codegen |

---

## 5. Standard Library Tests (tests/stdlib/)

### 5.1 Core Modules
| Test File | Module | Features |
|-----------|--------|----------|
| test_math.vp | math | sqrt, sin, cos, log, constants |
| test_random.vp | random | random(), randint(), choice() |
| test_time.vp | time | time(), sleep(), strftime() |
| test_datetime.vp | datetime | Date/time manipulation |
| test_re.vp | re | Regular expressions |
| test_json.vp | json | JSON parsing/generation |
| test_collections.vp | collections | deque, defaultdict, Counter |
| test_functools.vp | functools | partial, lru_cache, reduce |
| test_itertools.vp | itertools | Iterator utilities |

### 5.2 I/O Modules
| Test File | Module | Features |
|-----------|--------|----------|
| test_os.vp | os | Filesystem operations |
| test_io.vp | io | Stream operations |
| test_pathlib.vp | pathlib | Path manipulation |

### 5.3 Advanced Modules
| Test File | Module | Features |
|-----------|--------|----------|
| test_asyncio.vp | asyncio | Event loop, tasks |
| test_threading.vp | threading | Thread management |
| test_socket.vp | socket | Network sockets |
| test_http.vp | http | HTTP client/server |

---

## 6. Integration Tests (tests/integration/)

### 6.1 Algorithm Tests
| Test File | Algorithm | Description |
|-----------|-----------|-------------|
| test_sorting.vp | Sorting | QuickSort, MergeSort |
| test_searching.vp | Searching | Binary search, BFS, DFS |
| test_graph_algos.vp | Graph algorithms | Dijkstra, topological sort |
| test_dynamic_programming.vp | DP | Fibonacci, knapsack, LCS |

### 6.2 Real-world Scenarios
| Test File | Scenario | Description |
|-----------|----------|-------------|
| test_web_scraper.vp | Web scraper | HTTP + parsing |
| test_file_processor.vp | File processor | I/O + data processing |
| test_concurrent_pipeline.vp | Concurrent pipeline | Channels + tasks |
| test_api_client.vp | API client | Async HTTP + JSON |

### 6.3 Python Compatibility
| Test File | Feature Set | Description |
|-----------|-------------|-------------|
| test_python_builtins.vp | Built-in functions | len, range, map, filter, etc. |
| test_python_syntax.vp | Python syntax | Full Python 3 syntax compatibility |
| test_python_stdlib.vp | Python stdlib | Common stdlib patterns |

---

## 7. Benchmark Tests (tests/benchmarks/)

### 7.1 Performance Benchmarks
| Test File | Benchmark | Description |
|-----------|-----------|-------------|
| bench_prime.vp | Prime sieve | Computational performance |
| bench_fib.vp | Fibonacci | Recursion vs iteration |
| bench_matrix.vp | Matrix multiply | Numeric performance |
| bench_sort.vp | Sorting | Algorithm performance |

---

## Test Directory Structure

tests/
├── lexer/
│   ├── test_int_literals.vp
│   ├── test_float_literals.vp
│   ├── test_string_literals.vp
│   └── ...
├── parser/
│   ├── expressions/
│   │   ├── test_binary_ops.vp
│   │   └── ...
│   ├── statements/
│   │   ├── test_if.vp
│   │   └── ...
│   └── definitions/
│       ├── test_function_def.vp
│       └── ...
├── semantic/
│   ├── test_type_inference.vp
│   ├── test_type_annotations.vp
│   └── ...
├── codegen/
│   ├── test_arithmetic_codegen.vp
│   └── ...
├── stdlib/
│   ├── test_math.vp
│   ├── test_re.vp
│   └── ...
├── integration/
│   ├── test_sorting.vp
│   └── ...
└── benchmarks/
    ├── bench_prime.vp
    └── ...

---

## Test Execution

### Running Tests

Run all tests:
  cargo test

Run specific test category:
  cargo test lexer
  cargo test parser
  cargo test semantic
  cargo test codegen

Run with verbose output:
  cargo test -- --nocapture

Run single test file:
  cargo test --test test_int_literals

### Expected Test Output

Each test should:
1. Parse successfully (no syntax errors)
2. Type-check successfully (no type errors)
3. Generate valid LLVM IR
4. Execute and produce expected output
5. Clean up resources properly

---

## Coverage Goals

| Category | Target Coverage |
|----------|-----------------|
| Lexer | 100% token types |
| Parser | 100% grammar rules |
| Semantic | 95% type rules |
| CodeGen | 90% IR patterns |
| Stdlib | 80% public API |
| Integration | Key use cases |

---

## Notes

1. All test files use .vp extension
2. Tests should include expected output comments
3. Error tests should specify expected error messages
4. Performance tests should include timing constraints
5. Async tests should complete within timeout limits
