# Python Feature Testing Plan for Viper

## Goal
Test all Python features (language + stdlib) on Viper by running Viper code without modification, producing both summary and detailed reports, and continuing tests on failure.

## Configuration
- **Test Location**: `tests/02_python_compat/`
- **Test Runner**: `run_python_tests.sh` (new)
- **BigInt Tests**: Must PASS (full support expected)

## Output Formats

### Summary Report (`results/summary.txt`)
```
=== Python Feature Test Summary ===
Category              | Passed | Failed | Total
---------------------|--------|--------|-------
Data Types            |   7    |   1    |   8
...
TOTAL                 |  85    |   5    |  90
```

### Detailed Report (`results/detailed.json`)
```json
{
  "timestamp": "2026-03-03",
  "total": 90,
  "passed": 85,
  "failed": 5,
  "tests": [
    {"name": "test_int", "category": "data_types", "status": "PASS"},
    {"name": "test_factorial_big", "category": "math_library", "status": "PASS"},
    {"name": "test_json", "category": "stdlib", "status": "FAIL", "error": "..."}
  ]
}
```

## Test Categories (15 categories, ~100 test files)

| # | Category | Sub-features | Priority |
|---|----------|--------------|----------|
| 1 | Data Types | int, float, str, bool, list, dict, tuple, set, None, bytes | High |
| 2 | Operators | arithmetic, comparison, logical, bitwise, identity, membership | High |
| 3 | Control Flow | if/elif/else, for, while, match/case, break/continue/pass | High |
| 4 | Functions | def, *args, **kwargs, lambda, decorators, closures, generators | High |
| 5 | OOP | class, inheritance, methods, @property, @staticmethod, @classmethod | High |
| 6 | Exceptions | try/except/finally, raise, custom exceptions, exception chaining | High |
| 7 | Comprehensions | list, dict, set, generator expressions, nested | Medium |
| 8 | Context Managers | with statement, __enter__/__exit__ | Medium |
| 9 | Modules/Imports | import, from X import Y, stdlib modules | High |
| 10 | Builtin Functions | len, range, enumerate, zip, map, filter, sorted, etc. | High |
| 11 | String Operations | slicing, formatting, methods, f-strings | High |
| 12 | Collection Operations | list/dict/set/tuple methods and operations | High |
| 13 | Type Hints | basic types, Optional, Union, List, Dict, etc. | Medium |
| 14 | Math Library | all math functions, **bigint support** | High |
| 15 | Other Stdlib | json, random, collections, re, datetime, etc. | Medium |

## Directory Structure

```
tests/python_compat/
├── data_types/
│   ├── test_int.vp
│   ├── test_float.vp
│   ├── test_string.vp
│   ├── test_bool.vp
│   ├── test_list.vp
│   ├── test_dict.vp
│   ├── test_tuple.vp
│   ├── test_set.vp
│   └── test_none.vp
├── operators/
│   ├── test_arithmetic.vp
│   ├── test_comparison.vp
│   ├── test_logical.vp
│   └── test_bitwise.vp
├── control_flow/
│   ├── test_if_else.vp
│   ├── test_for_loop.vp
│   ├── test_while_loop.vp
│   ├── test_match_case.vp
│   └── test_break_continue.vp
├── functions/
│   ├── test_def.vp
│   ├── test_args_kwargs.vp
│   ├── test_lambda.vp
│   ├── test_decorator.vp
│   ├── test_closure.vp
│   └── test_generator.vp
├── oop/
│   ├── test_class.vp
│   ├── test_inheritance.vp
│   ├── test_property.vp
│   ├── test_staticmethod.vp
│   └── test_classmethod.vp
├── exceptions/
│   ├── test_try_except.vp
│   ├── test_raise.vp
│   └── test_finally.vp
├── comprehensions/
│   ├── test_list_comp.vp
│   ├── test_dict_comp.vp
│   └── test_set_comp.vp
├── context_managers/
│   └── test_with.vp
├── modules/
│   ├── test_import.vp
│   └── test_from_import.vp
├── builtin_functions/
│   ├── test_len_range.vp
│   ├── test_enumerate_zip.vp
│   └── test_map_filter.vp
├── string_operations/
│   ├── test_slicing.vp
│   ├── test_formatting.vp
│   ├── test_fstring.vp
│   └── test_string_methods.vp
├── collections/
│   ├── test_list_methods.vp
│   ├── test_dict_methods.vp
│   ├── test_set_methods.vp
│   └── test_tuple_methods.vp
├── type_hints/
│   └── test_basic_types.vp
├── math_library/
│   ├── test_math_basic.vp
│   ├── test_math_trig.vp
│   ├── test_math_log.vp
│   └── test_math_bigint.vp
└── stdlib/
    ├── test_json.vp
    ├── test_random.vp
    ├── test_collections.vp
    └── test_re.vp
```

## Test Runner Requirements

The `run_python_tests.sh` script must:

1. **Discover tests**: Find all `.vp` files in `tests/python_compat/`
2. **Run independently**: Each test runs separately
3. **Continue on failure**: Don't stop when a test fails
4. **Capture output**: Store stdout/stderr for each test
5. **Generate summary**: Table format with pass/fail per category
6. **Generate detailed**: JSON with full test results
7. **Handle timeouts**: Kill tests running >5 seconds

## Math Library + BigInt Priority

Critical bigint tests that must PASS:

```viper
# test_math_bigint.vp
import math

def test_factorial_big():
    result = math.factorial(1000)
    print(len(str(result)))  # 2568

def test_gcd_large():
    result = math.gcd(10**50, 10**40)
    print(result > 0)

def test_pow_large():
    result = pow(2, 1000)
    print(len(str(result)))  # 302

def test_sqrt_large():
    result = math.isqrt(10**100)
    print(result > 0)

def test_comb_perm():
    result = math.comb(100, 50)
    print(result > 0)
```

## Implementation Order

1. Create directory structure
2. Create test runner `run_python_tests.sh`
3. Create tests in priority order:
   - Phase 1: Data Types, Operators, Control Flow
   - Phase 2: Functions, OOP, Exceptions
   - Phase 3: Collections, Strings, Builtins
   - Phase 4: Math Library + BigInt (critical)
   - Phase 5: Stdlib, Comprehensions, etc.
4. Run and generate reports

## Success Criteria

- [ ] All tests run without crashing the test runner
- [ ] Summary report shows pass/fail per category
- [ ] Detailed report contains JSON with all test results
- [ ] Math bigint tests PASS
- [ ] Tests continue running after failures
