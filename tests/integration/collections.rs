//! Collection Integration Tests — lists, dicts, tuples, slices, list comprehensions

use crate::utils::run_viper_code;

// ============================================================================
// List Literals and Basic Operations
// ============================================================================

#[test]
fn test_list_empty() {
    let code = r#"
def test():
    lst = []
    print(len(lst))
test()
"#;
    let output = run_viper_code(code).expect("empty list should work");
    assert!(output.contains("0"), "got: {}", output);
}

#[test]
fn test_list_literal() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    print(len(lst))
test()
"#;
    let output = run_viper_code(code).expect("list literal should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_list_index_access() {
    let code = r#"
def test():
    lst = [10, 20, 30]
    print(lst[0])
    print(lst[1])
    print(lst[2])
test()
"#;
    let output = run_viper_code(code).expect("list index access should work");
    assert!(output.contains("10"), "got: {}", output);
    assert!(output.contains("20"), "got: {}", output);
    assert!(output.contains("30"), "got: {}", output);
}

#[test]
fn test_list_negative_index() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    print(lst[-1])
test()
"#;
    let output = run_viper_code(code).expect("negative index should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_list_append() {
    let code = r#"
def test():
    lst = []
    lst.append(1)
    lst.append(2)
    lst.append(3)
    print(len(lst))
test()
"#;
    let output = run_viper_code(code).expect("list append should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_list_pop() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    val = lst.pop()
    print(val)
    print(len(lst))
test()
"#;
    let output = run_viper_code(code).expect("list pop should work");
    assert!(output.contains("3"), "got: {}", output);
    assert!(output.contains("2"), "got: {}", output);
}

#[test]
fn test_list_assignment() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    lst[1] = 99
    print(lst[1])
test()
"#;
    let output = run_viper_code(code).expect("list item assignment should work");
    assert!(output.contains("99"), "got: {}", output);
}

#[test]
fn test_list_iteration_for() {
    let code = r#"
def test():
    lst = [10, 20, 30]
    total = 0
    for x in lst:
        total = total + x
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("list iteration should work");
    assert!(output.contains("60"), "got: {}", output);
}

#[test]
fn test_list_mixed_types() {
    let code = r#"
def test():
    lst = [1, "hello", True]
    print(len(lst))
test()
"#;
    let output = run_viper_code(code).expect("mixed type list should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_list_nested() {
    let code = r#"
def test():
    lst = [[1, 2], [3, 4]]
    print(lst[0][1])
    print(lst[1][0])
test()
"#;
    let output = run_viper_code(code).expect("nested list should work");
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// Slices
// ============================================================================

#[test]
fn test_slice_basic() {
    let code = r#"
def test():
    lst = [0, 1, 2, 3, 4]
    s = lst[1:3]
    print(len(s))
test()
"#;
    let output = run_viper_code(code).expect("basic slice should work");
    assert!(output.contains("2"), "got: {}", output);
}

#[test]
fn test_slice_from_start() {
    let code = r#"
def test():
    lst = [0, 1, 2, 3, 4]
    s = lst[:3]
    print(len(s))
test()
"#;
    let output = run_viper_code(code).expect("slice from start should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_slice_to_end() {
    let code = r#"
def test():
    lst = [0, 1, 2, 3, 4]
    s = lst[2:]
    print(len(s))
test()
"#;
    let output = run_viper_code(code).expect("slice to end should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_slice_with_step() {
    let code = r#"
def test():
    lst = [0, 1, 2, 3, 4]
    s = lst[::2]
    print(len(s))
test()
"#;
    let output = run_viper_code(code).expect("slice with step should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_slice_reverse() {
    let code = r#"
def test():
    lst = [1, 2, 3, 4, 5]
    s = lst[::-1]
    print(s[0])
test()
"#;
    let output = run_viper_code(code).expect("reverse slice should work");
    assert!(output.contains("5"), "got: {}", output);
}

#[test]
fn test_string_slice() {
    let code = r#"
def test():
    s = "hello"
    print(s[1:3])
test()
"#;
    let output = run_viper_code(code).expect("string slice should work");
    assert!(output.contains("el"), "got: {}", output);
}

// ============================================================================
// Dict Literals and Operations
// ============================================================================

#[test]
fn test_dict_empty() {
    let code = r#"
def test():
    d = {}
    print(len(d))
test()
"#;
    let output = run_viper_code(code).expect("empty dict should work");
    assert!(output.contains("0"), "got: {}", output);
}

#[test]
fn test_dict_literal() {
    let code = r#"
def test():
    d = {"a": 1, "b": 2}
    print(len(d))
test()
"#;
    let output = run_viper_code(code).expect("dict literal should work");
    assert!(output.contains("2"), "got: {}", output);
}

#[test]
fn test_dict_access() {
    let code = r#"
def test():
    d = {"name": "Alice", "age": 30}
    print(d["name"])
    print(d["age"])
test()
"#;
    let output = run_viper_code(code).expect("dict access should work");
    assert!(output.contains("Alice"), "got: {}", output);
    assert!(output.contains("30"), "got: {}", output);
}

#[test]
fn test_dict_set_item() {
    let code = r#"
def test():
    d = {}
    d["key"] = "value"
    print(d["key"])
test()
"#;
    let output = run_viper_code(code).expect("dict set item should work");
    assert!(output.contains("value"), "got: {}", output);
}

#[test]
fn test_dict_update() {
    let code = r#"
def test():
    d = {"x": 1}
    d["x"] = 99
    print(d["x"])
test()
"#;
    let output = run_viper_code(code).expect("dict update should work");
    assert!(output.contains("99"), "got: {}", output);
}

#[test]
fn test_dict_int_keys() {
    let code = r#"
def test():
    d = {1: "one", 2: "two"}
    print(d[1])
    print(d[2])
test()
"#;
    let output = run_viper_code(code).expect("dict int keys should work");
    assert!(output.contains("one"), "got: {}", output);
    assert!(output.contains("two"), "got: {}", output);
}

#[test]
fn test_dict_get_method() {
    let code = r#"
def test():
    d = {"x": 10}
    print(d.get("x"))
    print(d.get("missing"))
test()
"#;
    let output = run_viper_code(code).expect("dict.get should work");
    assert!(output.contains("10"), "got: {}", output);
}

#[test]
fn test_dict_in_operator() {
    let code = r#"
def test():
    d = {"a": 1, "b": 2}
    print("a" in d)
    print("c" in d)
test()
"#;
    let output = run_viper_code(code).expect("dict 'in' should work");
    assert!(output.contains("True"), "got: {}", output);
    assert!(output.contains("False"), "got: {}", output);
}

// ============================================================================
// Tuple Literals and Operations
// ============================================================================

#[test]
fn test_tuple_literal() {
    let code = r#"
def test():
    t = (1, 2, 3)
    print(len(t))
test()
"#;
    let output = run_viper_code(code).expect("tuple literal should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_tuple_index() {
    let code = r#"
def test():
    t = (10, 20, 30)
    print(t[0])
    print(t[2])
test()
"#;
    let output = run_viper_code(code).expect("tuple index should work");
    assert!(output.contains("10"), "got: {}", output);
    assert!(output.contains("30"), "got: {}", output);
}

#[test]
fn test_tuple_unpacking() {
    let code = r#"
def test():
    t = (1, 2, 3)
    a, b, c = t
    print(a)
    print(b)
    print(c)
test()
"#;
    let output = run_viper_code(code).expect("tuple unpacking should work");
    assert!(output.contains("1"), "got: {}", output);
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// List Comprehension
// ============================================================================

#[test]
fn test_list_comp_basic() {
    let code = r#"
def test():
    lst = [x * 2 for x in range(5)]
    print(len(lst))
    print(lst[0])
    print(lst[4])
test()
"#;
    let output = run_viper_code(code).expect("list comprehension should work");
    assert!(output.contains("5"), "got: {}", output);
    assert!(output.contains("0"), "got: {}", output);
    assert!(output.contains("8"), "got: {}", output);
}

#[test]
fn test_list_comp_sum() {
    let code = r#"
def test():
    lst = [i * i for i in range(1, 6)]
    total = 0
    for x in lst:
        total = total + x
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("list comp sum should work");
    assert!(output.contains("55"), "got: {}", output);
}

#[test]
fn test_list_comp_strings() {
    let code = r#"
def test():
    words = ["hello", "world", "viper"]
    lengths = [len(w) for w in words]
    print(lengths[0])
    print(lengths[1])
    print(lengths[2])
test()
"#;
    let output = run_viper_code(code).expect("list comp with strings should work");
    assert!(output.contains("5"), "got: {}", output);
    assert!(output.contains("5"), "got: {}", output);
    assert!(output.contains("5"), "got: {}", output);
}

// ============================================================================
// Membership Operators: in / not in
// ============================================================================

#[test]
fn test_in_operator_list() {
    let code = r#"
def test():
    lst = [1, 2, 3, 4, 5]
    print(3 in lst)
    print(99 in lst)
test()
"#;
    let output = run_viper_code(code).expect("'in' for list should work");
    assert!(output.contains("True"), "got: {}", output);
    assert!(output.contains("False"), "got: {}", output);
}

#[test]
fn test_not_in_operator_list() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    print(99 not in lst)
    print(1 not in lst)
test()
"#;
    let output = run_viper_code(code).expect("'not in' for list should work");
    assert!(output.contains("True"), "got: {}", output);
    assert!(output.contains("False"), "got: {}", output);
}

#[test]
fn test_in_operator_string() {
    let code = r#"
def test():
    s = "hello world"
    print("world" in s)
    print("xyz" in s)
test()
"#;
    let output = run_viper_code(code).expect("'in' for string should work");
    assert!(output.contains("True"), "got: {}", output);
    assert!(output.contains("False"), "got: {}", output);
}

#[test]
fn test_in_operator_in_if() {
    let code = r#"
def test():
    lst = [10, 20, 30]
    if 20 in lst:
        print("found")
    else:
        print("not found")
test()
"#;
    let output = run_viper_code(code).expect("'in' in if condition should work");
    assert!(output.contains("found"), "got: {}", output);
}
