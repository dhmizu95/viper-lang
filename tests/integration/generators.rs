//! Generator / Yield Integration Tests
//! Covers: yield statement, generator functions, for-in iteration over generators,
//!         lazy evaluation, generators with arguments

use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let _ = fs::remove_file(&test_file);
    if !output.status.success() {
        return Err(format!("stdout: {}\nstderr: {}", stdout, stderr));
    }
    Ok(stdout)
}

// ============================================================================
// Basic Generator Functions
// ============================================================================

#[test]
fn test_yield_simple() {
    let code = r#"
def simple_gen():
    yield 1
    yield 2
    yield 3

def test():
    for val in simple_gen():
        print(val)
test()
"#;
    let output = run_viper_code(code).expect("simple yield should work");
    assert!(output.contains("1"), "got: {}", output);
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_yield_in_loop() {
    let code = r#"
def counter(n):
    i = 0
    while i < n:
        yield i
        i = i + 1

def test():
    for val in counter(5):
        print(val)
test()
"#;
    let output = run_viper_code(code).expect("yield in loop should work");
    assert!(output.contains("0"), "got: {}", output);
    assert!(output.contains("4"), "got: {}", output);
}

#[test]
fn test_yield_with_range() {
    let code = r#"
def squares(n):
    for i in range(n):
        yield i * i

def test():
    for sq in squares(5):
        print(sq)
test()
"#;
    let output = run_viper_code(code).expect("yield with range should work");
    assert!(output.contains("0"), "got: {}", output);
    assert!(output.contains("1"), "got: {}", output);
    assert!(output.contains("4"), "got: {}", output);
    assert!(output.contains("9"), "got: {}", output);
    assert!(output.contains("16"), "got: {}", output);
}

#[test]
fn test_yield_sum_accumulation() {
    let code = r#"
def evens(n):
    for i in range(n):
        if i % 2 == 0:
            yield i

def test():
    total = 0
    for val in evens(10):
        total = total + val
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("yield sum should work");
    // evens(10) = 0+2+4+6+8 = 20
    assert!(output.contains("20"), "got: {}", output);
}

#[test]
fn test_yield_collect_to_list() {
    let code = r#"
def gen_ints(n):
    for i in range(n):
        yield i

def test():
    lst = []
    for v in gen_ints(5):
        lst.append(v)
    print(len(lst))
    print(lst[0])
    print(lst[4])
test()
"#;
    let output = run_viper_code(code).expect("yield to list should work");
    assert!(output.contains("5"), "got: {}", output);
    assert!(output.contains("0"), "got: {}", output);
    assert!(output.contains("4"), "got: {}", output);
}

#[test]
fn test_yield_with_string() {
    let code = r#"
def chars(s):
    for c in s:
        yield c

def test():
    result = ""
    for ch in chars("viper"):
        result = result + ch
    print(result)
test()
"#;
    let output = run_viper_code(code).expect("yield string chars should work");
    assert!(output.contains("viper"), "got: {}", output);
}

#[test]
fn test_yield_empty_generator() {
    let code = r#"
def empty():
    return
    yield 1  # unreachable, but makes it a generator

def test():
    count = 0
    for _ in empty():
        count = count + 1
    print(count)
test()
"#;
    let output = run_viper_code(code).expect("empty generator should work");
    assert!(output.contains("0"), "got: {}", output);
}

#[test]
fn test_yield_fibonacci() {
    let code = r#"
def fib_gen(n):
    a = 0
    b = 1
    count = 0
    while count < n:
        yield a
        a, b = b, a + b
        count = count + 1

def test():
    fibs = []
    for f in fib_gen(8):
        fibs.append(f)
    print(len(fibs))
    print(fibs[0])
    print(fibs[7])
test()
"#;
    let output = run_viper_code(code).expect("fibonacci generator should work");
    assert!(output.contains("8"), "got: {}", output);
    assert!(output.contains("0"), "got: {}", output);
    assert!(output.contains("13"), "got: {}", output);
}

// ============================================================================
// Generator Chaining / Composition
// ============================================================================

#[test]
fn test_yield_chained_generators() {
    let code = r#"
def first_n(gen, n):
    count = 0
    for val in gen:
        if count >= n:
            break
        yield val
        count = count + 1

def naturals():
    i = 0
    while True:
        yield i
        i = i + 1

def test():
    total = 0
    for v in first_n(naturals(), 5):
        total = total + v
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("chained generators should work");
    // 0+1+2+3+4 = 10
    assert!(output.contains("10"), "got: {}", output);
}
