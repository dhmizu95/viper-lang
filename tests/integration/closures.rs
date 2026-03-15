//! Closure and Advanced Function Integration Tests
//! Covers: default params, *args, nonlocal, global, closures, higher-order functions

use crate::utils::run_viper_code;

// ============================================================================
// Default Parameters
// ============================================================================

#[test]
fn test_default_param_single() {
    let code = r#"
def greet(name="World"):
    print("Hello " + name)

def test():
    greet()
    greet("Alice")
test()
"#;
    let output = run_viper_code(code).expect("default param should work");
    assert!(output.contains("Hello World"), "got: {}", output);
    assert!(output.contains("Hello Alice"), "got: {}", output);
}

#[test]
fn test_default_param_multiple() {
    let code = r#"
def add(a, b=10, c=100):
    return a + b + c

def test():
    print(add(1))
    print(add(1, 2))
    print(add(1, 2, 3))
test()
"#;
    let output = run_viper_code(code).expect("multiple default params should work");
    assert!(output.contains("111"), "got: {}", output);
    assert!(output.contains("103"), "got: {}", output);
    assert!(output.contains("6"), "got: {}", output);
}

#[test]
fn test_default_param_int() {
    let code = r#"
def repeat(n, times=3):
    i = 0
    total = 0
    while i < times:
        total = total + n
        i = i + 1
    return total

def test():
    print(repeat(5))
    print(repeat(5, 5))
test()
"#;
    let output = run_viper_code(code).expect("default int param should work");
    assert!(output.contains("15"), "got: {}", output);
    assert!(output.contains("25"), "got: {}", output);
}

#[test]
fn test_default_param_bool() {
    let code = r#"
def describe(x, verbose=False):
    if verbose:
        print("verbose: " + str(x))
    else:
        print(x)

def test():
    describe(42)
    describe(42, True)
test()
"#;
    let output = run_viper_code(code).expect("default bool param should work");
    assert!(output.contains("42"), "got: {}", output);
}

// ============================================================================
// Variadic Arguments (*args)
// ============================================================================

#[test]
fn test_variadic_args_sum() {
    let code = r#"
def sum_all(*nums):
    total = 0
    for n in nums:
        total = total + n
    return total

def test():
    print(sum_all(1, 2, 3))
    print(sum_all(10, 20))
    print(sum_all())
test()
"#;
    let output = run_viper_code(code).expect("variadic args should work");
    assert!(output.contains("6"), "got: {}", output);
    assert!(output.contains("30"), "got: {}", output);
    assert!(output.contains("0"), "got: {}", output);
}

#[test]
fn test_variadic_args_count() {
    let code = r#"
def count(*args):
    return len(args)

def test():
    print(count())
    print(count(1, 2, 3, 4, 5))
test()
"#;
    let output = run_viper_code(code).expect("variadic count should work");
    assert!(output.contains("0"), "got: {}", output);
    assert!(output.contains("5"), "got: {}", output);
}

#[test]
fn test_variadic_with_regular_param() {
    let code = r#"
def log(prefix, *messages):
    for msg in messages:
        print(prefix + msg)

def test():
    log("INFO: ", "started", "running")
test()
"#;
    let output = run_viper_code(code).expect("variadic with regular param should work");
    assert!(output.contains("INFO: started"), "got: {}", output);
    assert!(output.contains("INFO: running"), "got: {}", output);
}

// ============================================================================
// global Statement
// ============================================================================

#[test]
fn test_global_read() {
    let code = r#"
counter = 0

def get_counter():
    global counter
    return counter

def test():
    print(get_counter())
test()
"#;
    let output = run_viper_code(code).expect("global read should work");
    assert!(output.contains("0"), "got: {}", output);
}

#[test]
fn test_global_write() {
    let code = r#"
counter = 0

def increment():
    global counter
    counter = counter + 1

def test():
    increment()
    increment()
    increment()
    print(counter)
test()
"#;
    let output = run_viper_code(code).expect("global write should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_global_multiple_functions() {
    let code = r#"
value = 10

def double_value():
    global value
    value = value * 2

def triple_value():
    global value
    value = value * 3

def test():
    double_value()
    triple_value()
    print(value)
test()
"#;
    let output = run_viper_code(code).expect("multiple functions sharing global should work");
    assert!(output.contains("60"), "got: {}", output);
}

// ============================================================================
// nonlocal Statement
// ============================================================================

#[test]
fn test_nonlocal_basic() {
    let code = r#"
def outer():
    x = 10

    def inner():
        nonlocal x
        x = x + 5

    inner()
    return x

def test():
    print(outer())
test()
"#;
    let output = run_viper_code(code).expect("nonlocal should work");
    assert!(output.contains("15"), "got: {}", output);
}

#[test]
fn test_nonlocal_counter() {
    let code = r#"
def make_counter():
    count = 0

    def increment():
        nonlocal count
        count = count + 1
        return count

    return increment

def test():
    c = make_counter()
    print(c())
    print(c())
    print(c())
test()
"#;
    let output = run_viper_code(code).expect("nonlocal counter should work");
    assert!(output.contains("1"), "got: {}", output);
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// Closures — Capturing Outer Variables
// ============================================================================

#[test]
fn test_closure_captures_variable() {
    let code = r#"
def make_adder(n):
    def adder(x):
        return x + n
    return adder

def test():
    add5 = make_adder(5)
    add10 = make_adder(10)
    print(add5(3))
    print(add10(3))
test()
"#;
    let output = run_viper_code(code).expect("closure capture should work");
    assert!(output.contains("8"), "got: {}", output);
    assert!(output.contains("13"), "got: {}", output);
}

#[test]
fn test_closure_multiple_closures() {
    let code = r#"
def make_ops(n):
    def add(x):
        return x + n
    def mul(x):
        return x * n
    return add, mul

def test():
    add, mul = make_ops(4)
    print(add(6))
    print(mul(6))
test()
"#;
    let output = run_viper_code(code).expect("multiple closures should work");
    assert!(output.contains("10"), "got: {}", output);
    assert!(output.contains("24"), "got: {}", output);
}

// ============================================================================
// Higher-Order Functions
// ============================================================================

#[test]
fn test_higher_order_map() {
    let code = r#"
def apply_to_all(func, lst):
    result = []
    for x in lst:
        result.append(func(x))
    return result

def double(x):
    return x * 2

def test():
    nums = [1, 2, 3, 4, 5]
    doubled = apply_to_all(double, nums)
    print(doubled[0])
    print(doubled[4])
test()
"#;
    let output = run_viper_code(code).expect("higher-order map should work");
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("10"), "got: {}", output);
}

#[test]
fn test_higher_order_filter() {
    let code = r#"
def filter_list(pred, lst):
    result = []
    for x in lst:
        if pred(x):
            result.append(x)
    return result

def is_even(x):
    return x % 2 == 0

def test():
    nums = [1, 2, 3, 4, 5, 6]
    evens = filter_list(is_even, nums)
    print(len(evens))
    print(evens[0])
    print(evens[2])
test()
"#;
    let output = run_viper_code(code).expect("higher-order filter should work");
    assert!(output.contains("3"), "got: {}", output);
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("6"), "got: {}", output);
}

#[test]
fn test_function_as_return_value() {
    let code = r#"
def make_multiplier(factor):
    def multiply(x):
        return x * factor
    return multiply

def test():
    triple = make_multiplier(3)
    print(triple(7))
test()
"#;
    let output = run_viper_code(code).expect("function return value should work");
    assert!(output.contains("21"), "got: {}", output);
}

#[test]
fn test_lambda_as_argument() {
    let code = r#"
def apply(f, x):
    return f(x)

def test():
    print(apply(lambda x: x * x, 5))
    print(apply(lambda x: x + 1, 10))
test()
"#;
    let output = run_viper_code(code).expect("lambda as argument should work");
    assert!(output.contains("25"), "got: {}", output);
    assert!(output.contains("11"), "got: {}", output);
}

#[test]
fn test_lambda_with_closure() {
    let code = r#"
def make_adder(n):
    return lambda x: x + n

def test():
    add7 = make_adder(7)
    print(add7(3))
test()
"#;
    let output = run_viper_code(code).expect("lambda closure should work");
    assert!(output.contains("10"), "got: {}", output);
}
