# Integration Test Failures Analysis

## Summary
- **Master branch:** 202 tests, all passing
- **itg_test branch:** 354 tests (202 original + 152 new), 241 passing, **113 failing**

The 113 failing tests cover features that are either unimplemented or have runtime bugs in the Viper compiler.

---

## Failing Tests by Feature Category

### 1. Classes (12 tests)
Root cause: Runtime SIGSEGV crashes and codegen parameter type mismatches

- `test_class_dunder_str`
- `test_class_attribute_mutation`
- `test_class_init_and_attribute`
- `test_class_in_list`
- `test_class_method_chaining`
- `test_class_variable`
- `test_class_method_returns_value`
- `test_inheritance_inherits_methods`
- `test_class_str_method`
- `test_class_multiple_instances`
- `test_inheritance_super_init`
- `test_class_method_with_self`

**Error pattern:** `Call parameter type does not match function signature!`

---

### 2. Collections (17 tests)
Root cause: Runtime SIGSEGV crashes in list/dict operations

- `test_list_negative_index` (SIGSEGV)
- `test_list_comp_strings`
- `test_dict_access`
- `test_dict_get_method`
- `test_dict_int_keys`
- `test_in_operator_string`
- `test_dict_update`
- `test_dict_empty`
- `test_dict_set_item`
- `test_list_mixed_types`
- `test_list_nested`
- `test_dict_literal`
- `test_tuple_index`
- `test_list_comp_basic`
- `test_dict_in_operator`
- `test_slice_basic`
- `test_slice_reverse`
- `test_string_slice`
- `test_tuple_literal`

**Error patterns:**
- SIGSEGV (signal: 11)
- Misaligned pointer dereference
- Type mismatches in list operations

---

### 3. Exceptions (14 tests)
Root cause: Exception handling codegen issues

- `test_nested_try_except`
- `test_raise_in_function`
- `test_raise_reraise`
- `test_raise_propagation`
- `test_try_else_skipped_on_exception`
- `test_raise_simple`
- `test_try_except_catches_exception`
- `test_try_except_as_binding`
- `test_try_except_finally_all`
- `test_try_except_with_type`
- `test_try_finally_with_exception`
- `test_try_multiple_except`
- `test_try_in_loop`

**Error pattern:** Exceptions not caught or propagated correctly

---

### 4. Closures (12 tests)
Root cause: Codegen failure to capture closure variables

- `test_closure_captures_variable`
- `test_closure_multiple_closures`
- `test_default_param_bool`
- `test_default_param_multiple`
- `test_default_param_int`
- `test_default_param_single`
- `test_function_as_return_value`
- `test_higher_order_filter`
- `test_higher_order_map`
- `test_lambda_as_argument`
- `test_lambda_with_closure`
- `test_variadic_args_sum`
- `test_variadic_args_count`
- `test_nonlocal_counter`
- `test_variadic_with_regular_param`

**Error pattern:** `Codegen error: Undefined variable: adder`

---

### 5. With Statement (6 tests)
Root cause: Codegen parameter type mismatches for `__exit__` method

- `test_with_as_binding`
- `test_with_exit_called_on_success`
- `test_with_exit_called_on_exception`
- `test_with_body_executes`
- `test_with_basic_enter_exit`
- `test_with_return_value_from_enter`
- `test_with_nested`

**Error pattern:** `Call parameter type does not match function signature!` for `__exit__` calls

---

### 6. Match Statement (7 tests)
Root cause: Parser doesn't support match statement syntax

- `test_match_bool_constant` (Parser error: Unexpected token True)
- `test_match_string_constant` (SIGSEGV)
- `test_match_tuple_destructuring` (Codegen error)
- `test_match_guard_condition` (Codegen error)
- `test_match_tuple_pattern` (SIGSEGV)
- `test_match_wildcard_as_default` (Codegen error)
- `test_match_variable_binding` (SIGSEGV)

**Error patterns:**
- Parser: `Unexpected token in pattern`
- Codegen: Return type mismatches
- Runtime: SIGSEGV

---

### 7. Generators (8 tests)
Root cause: Parser doesn't support `yield` keyword

- `test_yield_empty_generator`
- `test_yield_chained_generators`
- `test_yield_fibonacci`
- `test_yield_collect_to_list`
- `test_yield_simple`
- `test_yield_with_string`
- `test_yield_sum_accumulation`
- `test_yield_in_loop`
- `test_yield_with_range`

**Error pattern:** `Driver error: Unexpected token in expression: Yield`

---

### 8. Imports (9 tests)
Root cause: Parser issues with list type annotations in stdlib

- `test_from_import_alias`
- `test_from_import_sqrt` (Type error: Undefined function sqrt)
- `test_from_import_used_directly`
- `test_import_alias`
- `test_import_math_floor` (Error: vp_math_ceil not declared)
- `test_from_import_multiple` (Type errors: Undefined floor, ceil)
- `test_import_math_pi` (Type errors)
- `test_import_sys` (Parser error in sys.vp: Expected Semi, found RBracket)
- `test_import_os_path` (Parser error in os.vp: Expected Semi, found RBracket)

**Error patterns:**
- Type errors for undefined functions
- Parser errors in stdlib modules

---

### 9. Concurrency (5 tests)
Root cause: Codegen parameter type mismatches

- `test_channel_basic_send_recv`
- `test_channel_multiple_messages`
- `test_channel_pipeline`
- `test_sync_runs_all_tasks`
- `test_waitgroup_basic`

**Error pattern:** `Call parameter type does not match function signature!`

---

### 10. Struct Types (9 tests)
Root cause: Parser doesn't support `struct` keyword

- `test_struct_arithmetic`
- `test_struct_as_return_value`
- `test_struct_in_list`
- `test_struct_float_fields`
- `test_struct_nested`
- `test_struct_basic_fields`
- `test_struct_multiple_instances`
- `test_type_alias_basic`
- `test_type_alias_for_int`

**Error pattern:** `Driver error: Expected identifier, found Colon`

---

### 11. Operators (5 tests)
Root cause: Missing operator implementation + runtime crashes

- `test_null_coalesce` (Parser error: Unexpected character '?')
- `test_pre_increment` (SIGSEGV)
- `test_post_decrement` (SIGSEGV)
- `test_pre_decrement` (SIGSEGV)
- `test_post_increment` (SIGSEGV)

---

## Root Cause Summary

| Category | Root Cause | Count |
|----------|-----------|-------|
| Parser | Missing `struct`, `match`, `yield`, return type annotations | ~30 |
| Runtime | SIGSEGV crashes in list/dict/indexing | ~25 |
| Codegen | Method/function call parameter type mismatches | ~25 |
| Codegen | Closure variable capture failures | ~15 |
| Stdlib | Missing math functions, broken module loading | ~10 |
| Runtime | Exception handling bugs | ~8 |

---

## Recommendations

To fix these tests, the following must be implemented:

1. **Parser**: Add support for `struct`, `match`, `yield`, return type annotations like `[str]`
2. **Runtime**: Fix SIGSEGV in list indexing, post-increment operators
3. **Codegen**: Fix closure capture, method call parameter types, exception handling
4. **Stdlib**: Implement missing math functions (sqrt, floor, ceil), fix module loading
