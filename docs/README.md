# Viper Language Documentation

Welcome to the Viper programming language documentation. This document provides an overview of all available documentation.

## Documentation Overview

| Document | Description |
|----------|-------------|
| [README.md](../README.md) | Project overview, quick start, and installation |
| [INSTALLATION.md](../INSTALLATION.md) | Detailed installation instructions |
| [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md) | Complete language syntax and features |
| [STDLIB_REFERENCE.md](STDLIB_REFERENCE.md) | Standard library API reference |
| [EXAMPLES_AND_BENCHMARKS.md](EXAMPLES_AND_BENCHMARKS.md) | Examples and benchmark suite |

---

## Quick Links

### Getting Started
- [Installation Guide](../INSTALLATION.md)
- [Quick Start](#quick-start)
- [Your First Viper Program](#your-first-viper-program)

### Language Reference
- [Syntax and Types](LANGUAGE_REFERENCE.md#language-syntax)
- [Control Flow](LANGUAGE_REFERENCE.md#control-flow)
- [Functions](LANGUAGE_REFERENCE.md#functions)
- [Collections](LANGUAGE_REFERENCE.md#collections)
- [Concurrency](LANGUAGE_REFERENCE.md#concurrency)
- [OOP](LANGUAGE_REFERENCE.md#object-oriented-programming)

### Standard Library
- [Core Modules](STDLIB_REFERENCE.md#core-modules)
- [Data Types](STDLIB_REFERENCE.md#data-types)
- [Networking](STDLIB_REFERENCE.md#networking)
- [Utilities](STDLIB_REFERENCE.md#utilities)

### Examples & Benchmarks
- [Basic Examples](EXAMPLES_AND_BENCHMARKS.md#basic-examples)
- [Algorithms](EXAMPLES_AND_BENCHMARKS.md#algorithm-examples)
- [Concurrency](EXAMPLES_AND_BENCHMARKS.md#concurrency-examples)
- [Benchmark Suite](EXAMPLES_AND_BENCHMARKS.md#benchmark-suite)

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/viper-lang/viper.git
cd viper

# Install dependencies and build
./install.sh

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
```

### Create Your First Program

```bash
# Create a new project
viper init hello
cd hello
```

This creates:
```
hello/
├── src/
│   └── main.vp
└── viper.json
```

Edit `src/main.vp`:

```python
# src/main.vp
def main():
    print("Hello, Viper!")

main()
```

### Run Your Program

```bash
# Run directly (JIT)
viper run src/main.vp
# Output: Hello, Viper!

# Build optimized binary
viper build src/main.vp -O 2 -o hello

# Run binary
./hello
# Output: Hello, Viper!
```

---

## Language Features

### Variables and Types

```python
# Type inference
x = 42
name = "Viper"
is_active = True

# Explicit types
age: i64 = 10
pi: f64 = 3.14159
```

### Control Flow

```python
# If/elif/else
if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")

# While loop
while i < 10:
    print(i)
    i = i + 1

# For loop
for i in range(10):
    print(i)
```

### Functions

```python
def greet(name: str) -> str:
    return "Hello, " + name + "!"

# Default parameters
def power(base, exp=2):
    return base ** exp

# Lambda functions
square = lambda x: x * x
```

### Collections

```python
# Lists
nums = [1, 2, 3, 4, 5]
nums.append(6)
first = nums[0]

# Dictionaries
person = {"name": "Alice", "age": 30}
name = person["name"]

# Arrays (fixed size, stack-allocated)
nums: [i64; 5] = [1, 2, 3, 4, 5]
```

### Concurrency

```python
# Channels
c = chan(10)
send(c, 42)
value = recv(c)

# Tasks
task worker(id, output):
    send(output, id * 2)

# WaitGroups
wg = WaitGroup()
wg.add(3)
wg.wait()
```

### Classes

```python
class Person:
    def __init__(self, name: str, age: i64):
        self.name = name
        self.age = age
    
    def greet(self) -> str:
        return "Hello, I am " + self.name

p = Person("Alice", 30)
```

---

## CLI Commands

### Build (AOT Compilation)

```bash
# Basic compilation
viper build program.vp -o program

# With optimization
viper build program.vp -O 2 -o program

# With LTO
viper build program.vp --lto -O 3 -o program
```

### Run (JIT Execution)

```bash
# Quick execution
viper run program.vp

# With optimization
viper run program.vp -O 2

# Debug mode
viper run program.vp --debug
```

### Other Commands

```bash
# Initialize new project
viper init myproject

# Format code
viper fmt

# Lint code
viper lint

# Show version
viper --version

# Show help
viper --help
```

---

## Project Structure

```
viper-lang/
├── docs/                    # Documentation
│   ├── LANGUAGE_REFERENCE.md
│   ├── STDLIB_REFERENCE.md
│   └── EXAMPLES_AND_BENCHMARKS.md
├── src/                     # Compiler source (Rust)
│   ├── ast/                 # AST definitions
│   ├── lexer/               # Lexical analysis
│   ├── parser/              # Parsing
│   ├── semantic/            # Type checking
│   ├── codegen/             # LLVM code generation
│   └── jit_stubs/           # JIT runtime stubs
├── runtime/                 # C runtime library
├── std/                     # Standard library (.vp)
├── benchmark/               # Benchmark suite
└── tests/                   # Test suite
```

---

## Additional Resources

### Existing Documentation

| File | Description |
|------|-------------|
| [AGENTS.md](../AGENTS.md) | Developer guidelines |
| [PROJECT_OVERVIEW.md](../PROJECT_OVERVIEW.md) | Detailed project overview |
| [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md) | Codebase structure |
| [CORE_LANGUAGE_FEATURES.md](../CORE_LANGUAGE_FEATURES.md) | Language feature details |

### Performance

| File | Description |
|------|-------------|
| [OPTIMIZATIONS.md](../OPTIMIZATIONS.md) | Optimization techniques |
| [OPTIMIZATION_STATUS.md](../OPTIMIZATION_STATUS.md) | Implementation status |
| [benchmark/RESULTS.md](../benchmark/RESULTS.md) | Benchmark results |

### Advanced Topics

| File | Description |
|------|-------------|
| [FIBER_SCHEDULER.md](../FIBER_SCHEDULER.md) | Concurrency scheduler |
| [ARC.md](../ARC.md) | Reference counting |
| [PYTHON_COMPATIBILITY_ROADMAP.md](../PYTHON_COMPATIBILITY_ROADMAP.md) | Python compatibility |

---

## Version Information

- **Current Version**: 0.4.0
- **License**: MIT
- **Repository**: github.com/viper-lang/viper

---

## Contributing

Contributions are welcome! Please see [AGENTS.md](../AGENTS.md) for development guidelines and coding standards.

---

*Last updated: 2024*
