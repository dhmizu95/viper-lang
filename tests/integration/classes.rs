//! OOP / Class Integration Tests
//! Covers: class definition, __init__, instance methods, attributes,
//!         inheritance, super(), method override

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
// Basic Class Definition
// ============================================================================

#[test]
fn test_class_simple_definition() {
    let code = r#"
class Dog:
    def bark(self):
        print("Woof!")

def test():
    d = Dog()
    d.bark()
test()
"#;
    let output = run_viper_code(code).expect("simple class should work");
    assert!(output.contains("Woof!"), "got: {}", output);
}

#[test]
fn test_class_init_and_attribute() {
    let code = r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

def test():
    p = Point(3, 4)
    print(p.x)
    print(p.y)
test()
"#;
    let output = run_viper_code(code).expect("class __init__ should work");
    assert!(output.contains("3"), "got: {}", output);
    assert!(output.contains("4"), "got: {}", output);
}

#[test]
fn test_class_method_with_self() {
    let code = r#"
class Counter:
    def __init__(self):
        self.count = 0

    def increment(self):
        self.count = self.count + 1

    def get(self):
        return self.count

def test():
    c = Counter()
    c.increment()
    c.increment()
    c.increment()
    print(c.get())
test()
"#;
    let output = run_viper_code(code).expect("class methods should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_class_method_returns_value() {
    let code = r#"
class Rectangle:
    def __init__(self, w, h):
        self.w = w
        self.h = h

    def area(self):
        return self.w * self.h

    def perimeter(self):
        return 2 * (self.w + self.h)

def test():
    r = Rectangle(5, 3)
    print(r.area())
    print(r.perimeter())
test()
"#;
    let output = run_viper_code(code).expect("class value methods should work");
    assert!(output.contains("15"), "got: {}", output);
    assert!(output.contains("16"), "got: {}", output);
}

#[test]
fn test_class_multiple_instances() {
    let code = r#"
class Box:
    def __init__(self, val):
        self.val = val

def test():
    a = Box(10)
    b = Box(20)
    c = Box(30)
    print(a.val + b.val + c.val)
test()
"#;
    let output = run_viper_code(code).expect("multiple instances should work");
    assert!(output.contains("60"), "got: {}", output);
}

#[test]
fn test_class_attribute_mutation() {
    let code = r#"
class Account:
    def __init__(self, balance):
        self.balance = balance

    def deposit(self, amount):
        self.balance = self.balance + amount

    def withdraw(self, amount):
        self.balance = self.balance - amount

def test():
    acc = Account(100)
    acc.deposit(50)
    acc.withdraw(30)
    print(acc.balance)
test()
"#;
    let output = run_viper_code(code).expect("attribute mutation should work");
    assert!(output.contains("120"), "got: {}", output);
}

#[test]
fn test_class_str_method() {
    let code = r#"
class Person:
    def __init__(self, name, age):
        self.name = name
        self.age = age

    def describe(self):
        print(self.name + " is " + str(self.age))

def test():
    p = Person("Alice", 30)
    p.describe()
test()
"#;
    let output = run_viper_code(code).expect("class str method should work");
    assert!(output.contains("Alice is 30"), "got: {}", output);
}

// ============================================================================
// Inheritance
// ============================================================================

#[test]
fn test_inheritance_basic() {
    let code = r#"
class Animal:
    def speak(self):
        print("...")

class Dog(Animal):
    def speak(self):
        print("Woof")

class Cat(Animal):
    def speak(self):
        print("Meow")

def test():
    d = Dog()
    c = Cat()
    d.speak()
    c.speak()
test()
"#;
    let output = run_viper_code(code).expect("basic inheritance should work");
    assert!(output.contains("Woof"), "got: {}", output);
    assert!(output.contains("Meow"), "got: {}", output);
}

#[test]
fn test_inheritance_inherits_methods() {
    let code = r#"
class Vehicle:
    def __init__(self, speed):
        self.speed = speed

    def describe(self):
        print(self.speed)

class Car(Vehicle):
    def __init__(self, speed, brand):
        self.speed = speed
        self.brand = brand

def test():
    c = Car(120, "Tesla")
    c.describe()
    print(c.brand)
test()
"#;
    let output = run_viper_code(code).expect("inherited method should work");
    assert!(output.contains("120"), "got: {}", output);
    assert!(output.contains("Tesla"), "got: {}", output);
}

#[test]
fn test_inheritance_super_init() {
    let code = r#"
class Animal:
    def __init__(self, name):
        self.name = name

class Dog(Animal):
    def __init__(self, name, breed):
        super().__init__(name)
        self.breed = breed

def test():
    d = Dog("Rex", "Labrador")
    print(d.name)
    print(d.breed)
test()
"#;
    let output = run_viper_code(code).expect("super().__init__ should work");
    assert!(output.contains("Rex"), "got: {}", output);
    assert!(output.contains("Labrador"), "got: {}", output);
}

#[test]
fn test_inheritance_super_method() {
    let code = r#"
class Shape:
    def area(self):
        return 0

    def describe(self):
        print("Area:", self.area())

class Circle(Shape):
    def __init__(self, r):
        self.r = r

    def area(self):
        return self.r * self.r

def test():
    c = Circle(5)
    c.describe()
test()
"#;
    let output = run_viper_code(code).expect("super method dispatch should work");
    assert!(output.contains("25"), "got: {}", output);
}

#[test]
fn test_inheritance_multi_level() {
    let code = r#"
class A:
    def hello(self):
        print("A")

class B(A):
    def hello(self):
        print("B")

class C(B):
    def hello(self):
        print("C")

def test():
    a = A()
    b = B()
    c = C()
    a.hello()
    b.hello()
    c.hello()
test()
"#;
    let output = run_viper_code(code).expect("multi-level inheritance should work");
    assert!(output.contains("A"), "got: {}", output);
    assert!(output.contains("B"), "got: {}", output);
    assert!(output.contains("C"), "got: {}", output);
}

#[test]
fn test_class_in_list() {
    let code = r#"
class Item:
    def __init__(self, value):
        self.value = value

def test():
    items = [Item(1), Item(2), Item(3)]
    total = 0
    for item in items:
        total = total + item.value
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("list of class instances should work");
    assert!(output.contains("6"), "got: {}", output);
}

#[test]
fn test_class_method_chaining() {
    let code = r#"
class Builder:
    def __init__(self):
        self.parts = []

    def add(self, part):
        self.parts.append(part)
        return self

    def count(self):
        return len(self.parts)

def test():
    b = Builder()
    b.add("a")
    b.add("b")
    b.add("c")
    print(b.count())
test()
"#;
    let output = run_viper_code(code).expect("chained method calls should work");
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// Class with __str__ / __repr__
// ============================================================================

#[test]
fn test_class_dunder_str() {
    let code = r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __str__(self):
        return "(" + str(self.x) + ", " + str(self.y) + ")"

def test():
    p = Point(1, 2)
    print(str(p))
test()
"#;
    let output = run_viper_code(code).expect("__str__ should work");
    assert!(output.contains("(1, 2)"), "got: {}", output);
}

// ============================================================================
// Class Variables
// ============================================================================

#[test]
fn test_class_variable() {
    let code = r#"
class Counter:
    total = 0

    def __init__(self):
        Counter.total = Counter.total + 1

def test():
    a = Counter()
    b = Counter()
    c = Counter()
    print(Counter.total)
test()
"#;
    let output = run_viper_code(code).expect("class variable should work");
    assert!(output.contains("3"), "got: {}", output);
}
