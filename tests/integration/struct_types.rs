//! Struct and Type Alias Integration Tests
//! Covers: struct definition, instantiation, field access, type alias

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
// Struct Definition and Field Access
// ============================================================================

#[test]
fn test_struct_basic_fields() {
    let code = r#"
struct Point:
    x: i64
    y: i64

def test():
    p = Point(3, 4)
    print(p.x)
    print(p.y)
test()
"#;
    let output = run_viper_code(code).expect("struct field access should work");
    assert!(output.contains("3"), "got: {}", output);
    assert!(output.contains("4"), "got: {}", output);
}

#[test]
fn test_struct_arithmetic() {
    let code = r#"
struct Vec2:
    x: i64
    y: i64

def length_squared(v: Vec2) -> i64:
    return v.x * v.x + v.y * v.y

def test():
    v = Vec2(3, 4)
    print(length_squared(v))
test()
"#;
    let output = run_viper_code(code).expect("struct in function should work");
    assert!(output.contains("25"), "got: {}", output);
}

#[test]
fn test_struct_multiple_instances() {
    let code = r#"
struct Color:
    r: i64
    g: i64
    b: i64

def test():
    red = Color(255, 0, 0)
    green = Color(0, 255, 0)
    blue = Color(0, 0, 255)
    print(red.r)
    print(green.g)
    print(blue.b)
test()
"#;
    let output = run_viper_code(code).expect("multiple struct instances should work");
    assert!(output.contains("255"), "got: {}", output);
}

#[test]
fn test_struct_float_fields() {
    let code = r#"
struct Coord:
    lat: float
    lon: float

def test():
    loc = Coord(1.5, 2.5)
    print(loc.lat)
    print(loc.lon)
test()
"#;
    let output = run_viper_code(code).expect("struct float fields should work");
    assert!(output.contains("1.5"), "got: {}", output);
    assert!(output.contains("2.5"), "got: {}", output);
}

#[test]
fn test_struct_in_list() {
    let code = r#"
struct Item:
    value: i64

def test():
    items = [Item(10), Item(20), Item(30)]
    total = 0
    for item in items:
        total = total + item.value
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("list of structs should work");
    assert!(output.contains("60"), "got: {}", output);
}

#[test]
fn test_struct_as_return_value() {
    let code = r#"
struct Pair:
    first: i64
    second: i64

def make_pair(a: i64, b: i64) -> Pair:
    return Pair(a, b)

def test():
    p = make_pair(7, 13)
    print(p.first)
    print(p.second)
test()
"#;
    let output = run_viper_code(code).expect("struct as return value should work");
    assert!(output.contains("7"), "got: {}", output);
    assert!(output.contains("13"), "got: {}", output);
}

#[test]
fn test_struct_nested() {
    let code = r#"
struct Inner:
    val: i64

struct Outer:
    inner: Inner
    extra: i64

def test():
    i = Inner(42)
    o = Outer(i, 10)
    print(o.inner.val)
    print(o.extra)
test()
"#;
    let output = run_viper_code(code).expect("nested structs should work");
    assert!(output.contains("42"), "got: {}", output);
    assert!(output.contains("10"), "got: {}", output);
}

// ============================================================================
// Type Alias
// ============================================================================

#[test]
fn test_type_alias_basic() {
    let code = r#"
type Name = str

def greet(name: Name):
    print("Hello " + name)

def test():
    greet("Alice")
test()
"#;
    let output = run_viper_code(code).expect("type alias should work");
    assert!(output.contains("Hello Alice"), "got: {}", output);
}

#[test]
fn test_type_alias_for_int() {
    let code = r#"
type Count = i64

def double(x: Count) -> Count:
    return x * 2

def test():
    print(double(21))
test()
"#;
    let output = run_viper_code(code).expect("type alias for int should work");
    assert!(output.contains("42"), "got: {}", output);
}
