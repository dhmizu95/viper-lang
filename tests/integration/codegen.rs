//! Code Generation Integration Tests

use inkwell::context::Context;
use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use viper_lang::codegen::CodeGen;
use viper_lang::lexer::Lexer;
use viper_lang::parser::Parser;

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

fn generate_module_ir(code: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "test_module");
    codegen.generate(&ast).map_err(|e| e.to_string())?;
    codegen.verify().map_err(|e| e.to_string())?;

    Ok(codegen.module().print_to_string().to_string())
}

// Arithmetic Codegen
#[test]
fn test_codegen_arithmetic_int() {
    let code = r#"
def test():
    a = 10 + 5 * 2
    b = (10 + 5) * 2
    c = 100 / 4
    d = 17 % 5
    print(a)
    print(b)
    print(c)
    print(d)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("20"));
    assert!(output.contains("30"));
    assert!(output.contains("25"));
    assert!(output.contains("2"));
}

#[test]
fn test_codegen_arithmetic_float() {
    let code = r#"
def test():
    a = 3.14 + 2.86
    b = 10.0 / 4.0
    print(a)
    print(b)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("6"));
    assert!(output.contains("2.5"));
}

// Comparison Codegen
#[test]
fn test_codegen_comparison() {
    let code = r#"
def test():
    print(5 < 10)
    print(5 > 10)
    print(5 == 5)
    print(5 != 10)
    print(5 <= 5)
    print(5 >= 5)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("True"));
    assert!(output.contains("False"));
}

// Logical Codegen
#[test]
fn test_codegen_logical() {
    let code = r#"
def test():
    print(True and False)
    print(True or False)
    print(not True)
    print(not False)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("False"));
}

// Branch Codegen
#[test]
fn test_codegen_branches_if() {
    let code = r#"
def test():
    x = 10
    if x > 5:
        print("branch1")
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("branch1"));
}

#[test]
fn test_codegen_branches_if_else() {
    let code = r#"
def test():
    x = 3
    if x > 5:
        print("greater")
    else:
        print("less")
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("less"));
}

// Loop Codegen
#[test]
fn test_codegen_loops_while() {
    let code = r#"
def test():
    i = 0
    while i < 5:
        i = i + 1
    print(i)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("\n5\n") || output.ends_with("5\n"));
}

#[test]
fn test_codegen_loops_nested() {
    let code = r#"
def test():
    i = 0
    j = 0
    while i < 3:
        while j < 3:
            j = j + 1
        i = i + 1
        j = 0
    print(i)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("\n3\n") || output.ends_with("3\n"));
}

// Function Codegen
#[test]
fn test_codegen_functions_simple() {
    let code = r#"
def add(a, b):
    return a + b

def test():
    print(add(10, 20))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("30"));
}

#[test]
fn test_codegen_functions_recursive() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    print(factorial(5))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_codegen_task_global_aug_assign_is_atomic() {
    let code = r#"
counter = 0

def bump():
    global counter
    counter += 1

def test():
    sync:
        task bump()
"#;

    let ir = generate_module_ir(code).unwrap();
    assert!(
        ir.contains("atomicrmw add"),
        "expected shared global increment to lower to atomicrmw, got:\n{}",
        ir
    );
}

#[test]
fn test_codegen_global_aug_assign_preserves_rhs_evaluation_order() {
    let code = r#"
counter = 1

def rhs():
    global counter
    counter = 10
    return 2

def test():
    global counter
    counter += rhs()
    print(counter)
test()
"#;

    let output = run_viper_code(code).unwrap();
    assert!(output.contains("\n3\n") || output.ends_with("3\n"), "unexpected output: {}", output);
}

// Closure Codegen
#[test]
fn test_codegen_closures() {
    let code = r#"
def test():
    f = lambda x: x * 2
    print(f(21))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("42"));
}
