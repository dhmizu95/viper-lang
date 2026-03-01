# Viper Language Reference

A comprehensive reference guide for the Viper programming language.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Language Syntax](#language-syntax)
4. [Data Types](#data-types)
5. [Control Flow](#control-flow)
6. [Functions](#functions)
7. [Collections](#collections)
8. [Concurrency](#concurrency)
9. [Object-Oriented Programming](#object-oriented-programming)
10. [Error Handling](#error-handling)
11. [Standard Library](#standard-library)
12. [Examples](#examples)

---

## Introduction

Viper is a high-performance programming language that compiles to native code using LLVM. It combines the expressiveness of Python with the performance of C.

### Key Features

- **LLVM-based compilation**: Generates highly optimized native code
- **JIT and AOT**: Supports both just-in-time and ahead-of-time compilation
- **Automatic Reference Counting (ARC)**: Automatic memory management
- **Escape Analysis**: Optimizes stack vs heap allocation
- **Structured Concurrency**: Built-in support for channels, tasks, and async/await
- **Python-like Syntax**: Familiar and easy to learn
- **Zero Dependencies**: Compiled binaries are fully static

---

## Getting Started

### Installation

```bash
git clone https://github.com/viper-lang/viper.git
cd viper
./install.sh
export PATH="$HOME/.local/bin:$PATH"
```

### Quick Start

```bash
# Create a new project
viper init myproject
cd myproject

# Run the program
viper run src/main.vp

# Build optimized binary
viper build src/main.vp -O 2 -o myapp
```

---

## Language Syntax

### Variables

```python
# Type inference (implicit)
x = 42
name = "Viper"
is_active = True

# Explicit type annotation
x: i64 = 42
name: str = "Viper"
```

### Comments

```python
# This is a single-line comment

# This is a
    multi-line
    comment
```

---

## Data Types

### Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `i64` | 64-bit signed integer | `42` |
| `f64` | 64-bit floating point | `3.14` |
| `bool` | Boolean | `True`, `False` |
| `str` | String | `"Hello"` |
| `None` | Null value | `None` |

### BigInt Support

```python
# Using 'n' suffix
x = 123456789012345678901234567890n

# Automatic promotion (too large for i64)
y = 999999999999999999999999999999

# BigInt constructor
z = BigInt("123456789012345678901234567890")

# Operations
a = 100n + 200n
b = 50n * 2n
c = 10n ** 100n
```

### Arrays (Fixed Size)

```python
# Stack-allocated arrays for performance
nums: [i64; 5] = [1, 2, 3, 4, 5]

# Array with repetition
zeros = [0; 100]  # [0, 0, 0, ...] (100 zeros)
```

### Lists (Dynamic)

```python
# Create empty list
nums = []

# Create with values
nums = [1, 2, 3, 4, 5]

# Pre-allocate for performance
nums = [0] * 1000000  # Fast allocation

# Access elements
first = nums[0]
last = nums[len(nums) - 1]

# Modify
nums[0] = 10
nums.append(6)
nums.pop()
```

### Dictionaries

```python
# Create dictionary
d = {}

# With values
person = {"name": "Alice", "age": 30}

# Access
name = person["name"]

# Modify
person["age"] = 31
person["city"] = "NYC"
```

### Tuples

```python
# Create tuple
point = (10, 20)

# Access
x = point[0]
y = point[1]

# Multiple return values
def divide(a, b):
    return (a / b, a % b)
```

---

## Control Flow

### If/Else

```python
x = 10

if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")
```

### While Loop

```python
i = 0
while i < 10:
    print(i)
    i = i + 1
```

### For Loop

```python
# Range iteration
for i in range(10):  # 0 to 9
    print(i)

for i in range(5, 10):  # 5 to 9
    print(i)

for i in range(0, 10, 2):  # 0, 2, 4, 6, 8
    print(i)

# List iteration
for item in my_list:
    print(item)
```

### Match (Pattern Matching)

```python
value = 42

match value:
    case 0:
        print("zero")
    case 1 | 2 | 3:
        print("one, two, or three")
    case _:
        print("other")
```

---

## Functions

### Basic Functions

```python
def greet(name: str) -> str:
    return "Hello, " + name + "!"

# Call function
result = greet("Viper")
```

### Default Parameters

```python
def power(base, exp=2):
    result = 1
    for i in range(exp):
        result = result * base
    return result

# Usage
square = power(4)      # 16
cube = power(4, 3)     # 64
```

### Variadic Functions

```python
def sum_all(*args):
    total = 0
    for n in args:
        total = total + n
    return total

# Usage
result = sum_all(1, 2, 3, 4, 5)  # 15
```

### Lambda Functions

```python
# Anonymous function
square = lambda x: x * x

# With multiple parameters
add = lambda a, b: a + b
```

### Generator Functions

```python
def count_to(n):
    i = 0
    while i < n:
        yield i
        i = i + 1

# Usage
for num in count_to(5):
    print(num)  # 0, 1, 2, 3, 4
```

---

## Collections

### List Methods

```python
nums = [1, 2, 3, 4, 5]

# Basic operations
len(nums)              # Length: 5
nums.append(6)         # Add: [1,2,3,4,5,6]
nums.pop()             # Remove last: 6
nums.insert(0, 0)     # Insert at index: [0,1,2,3,4,5]
nums.remove(3)         # Remove first 3: [0,1,2,4,5]

# Search
idx = nums.index(4)   # Index of 4: 3
found = 5 in nums      # Contains: True

# Sorting
sorted_nums = sorted(nums)        # New sorted list
nums.sort()                         # Sort in place

# Reversed
reversed_nums = reversed(nums)      # Iterator
nums.reverse()                      # In place
```

### List Comprehensions

```python
# Create list from range
squares = [x * x for x in range(10)]

# With condition
evens = [x for x in range(20) if x % 2 == 0]

# Nested
matrix = [[i * j for j in range(5)] for i in range(5)]
```

### Dictionary Operations

```python
d = {"a": 1, "b": 2, "c": 3}

# Access
val = d["a"]           # 1
val = d.get("d", 0)    # 0 (default)

# Modify
d["d"] = 4             # Add
del d["a"]             # Remove

# Methods
keys = d.keys()        # ["a", "b", "c", "d"]
values = d.values()    # [1, 2, 3, 4]
items = d.items()     # [("a",1), ("b",2), ...]

# Dictionary comprehension
squares = {x: x*x for x in range(10)}
```

---

## Concurrency

### Channels

```python
# Create channel
c = chan(10)  # Buffer size 10

# Send values
send(c, 42)

# Receive values
value = recv(c)
```

### Tasks

```python
# Spawn concurrent task
task worker(id: i64, out_chan):
    # Task body
    send(out_chan, id * 2)

# Create channel for results
result_chan = chan(10)

# Spawn multiple workers
for i in range(5):
    task worker(i, result_chan)

# Collect results
for i in range(5):
    result = recv(result_chan)
```

### WaitGroups

```python
# Synchronize multiple tasks
wg = WaitGroup()

# Add task count
wg.add(3)

# Spawn tasks
task do_work(wg):
    # Do work
    wg.done()  # Signal completion

# Wait for all
wg.wait()
```

### Select Statement

```python
select:
    case v = recv(chan1):
        print("received from chan1:", v)
    case chan2 <- 42:
        print("sent to chan2")
    case timeout(1000):
        print("timeout")
```

### Async/Await

```python
# Define async function
async def fetch(url: str) -> str:
    # Async operations
    return "data"

# Run async
async def main():
    result = await fetch("https://example.com")
    print(result)

# Execute
asyncio.run(main())
```

---

## Object-Oriented Programming

### Classes

```python
class Person:
    # Constructor
    def __init__(self, name: str, age: i64):
        self.name = name
        self.age = age
    
    # Method
    def greet(self) -> str:
        return "Hello, I am " + self.name
    
    # Property
    def is_adult(self) -> bool:
        return self.age >= 18

# Usage
p = Person("Alice", 30)
print(p.greet())        # "Hello, I am Alice"
print(p.is_adult())     # True
```

### Inheritance

```python
class Animal:
    def __init__(self, name: str):
        self.name = name
    
    def speak(self) -> str:
        return ""

class Dog(Animal):
    def speak(self) -> str:
        return self.name + " says woof!"

class Cat(Animal):
    def speak(self) -> str:
        return self.name + " says meow!"

# Usage
dog = Dog("Buddy")
print(dog.speak())  # "Buddy says woof!"
```

### Generics

```python
class Stack[T]:
    def __init__(self):
        self.items = []
    
    def push(self, item: T):
        self.items.append(item)
    
    def pop(self) -> T:
        return self.items.pop()
    
    def peek(self) -> T:
        return self.items[len(self.items) - 1]

# Usage
int_stack = Stack[i64]()
int_stack.push(42)
int_stack.push(100)

str_stack = Stack[str]()
str_stack.push("hello")
str_stack.push("world")
```

---

## Error Handling

### Result Type

```python
# Function returning Result
def divide(a: i64, b: i64) -> Result[i64, str]:
    if b == 0:
        return Err("division by zero")
    return Ok(a / b)

# Using Result
result = divide(10, 2)

if result.is_ok():
    value = result.unwrap()
    print("Result:", value)
else:
    error = result.unwrap_err()
    print("Error:", error)
```

### Using ? Operator

```python
def compute() -> Result[i64, str]:
    a = try!(divide(10, 2))
    b = try!(divide(a, 2))
    return Ok(b)

# Or with ? syntax (panics on error)
def compute() -> Result[i64, str]:
    a = divide(10, 2)?
    b = divide(a, 2)?
    return Ok(b)
```

### unwrap_or

```python
def safe_divide(a: i64, b: i64) -> i64:
    result = divide(a, b)
    return result.unwrap_or(0)  # Default on error
```

### Try/Catch (Future)

```python
# Coming soon
try:
    risky_operation()
except Error as e:
    print("Caught error:", e)
finally:
    cleanup()
```

---

## Standard Library

### Core Modules

| Module | Description |
|--------|-------------|
| `os` | Operating system interface |
| `sys` | System-specific parameters |
| `time` | Time and date functions |
| `math` | Mathematical functions |
| `json` | JSON serialization |
| `http` | HTTP client/server |
| `socket` | Network sockets |
| `random` | Random number generation |
| `hashlib` | Cryptographic hashing |
| `collections` | Collection utilities |

### Built-in Functions

```python
# Print output
print("Hello, World!")

# Length
len([1, 2, 3])        # 3
len("hello")          # 5

# Type checking
typeof(42)            # "i64"
typeof("hello")       # "str"

# Range
range(10)            # 0-9
range(5, 10)         # 5-9
range(0, 10, 2)      # 0,2,4,6,8

# Type conversion
int("42")            # 42
str(42)              # "42"
float("3.14")        # 3.14
bool(1)              # True

# Min/Max
min(1, 2, 3)         # 1
max(1, 2, 3)         # 3

# Absolute value
abs(-42)             # 42
```

---

## Examples

### Hello World

```python
# hello.vp
print("Hello, World!")
```

### Factorial

```python
# factorial.vp
def factorial(n: i64) -> i64:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

print("5! =", factorial(5))  # 120
```

### Fibonacci

```python
# fibonacci.vp
def fib(n: i64) -> i64:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

for i in range(10):
    print("fib(", i, ") =", fib(i))
```

### Prime Sieve

```python
# sieve.vp
def sieve(n: i64) -> i64:
    is_prime = [True] * (n + 1)
    is_prime[0] = False
    is_prime[1] = False
    
    for i in range(2, int(sqrt(n)) + 1):
        if is_prime[i]:
            for j in range(i * i, n + 1, i):
                is_prime[j] = False
    
    count = 0
    for i in range(2, n + 1):
        if is_prime[i]:
            count = count + 1
    
    return count

print("Primes below 100000:", sieve(100000))
```

### Concurrent Worker Pool

```python
# worker_pool.vp
def worker(id: i64, jobs: chan[i64], results: chan[i64]):
    while True:
        select:
            case job = recv(jobs):
                result = job * 2  # Process job
                send(results, result)
            case timeout(5000):
                print("Worker", id, "timing out")
                break

def main():
    jobs = chan(100)
    results = chan(100)
    
    # Start workers
    for i in range(5):
        task worker(i, jobs, results)
    
    # Send jobs
    for i in range(20):
        send(jobs, i)
    
    # Close jobs channel
    close(jobs)
    
    # Collect results
    for i in range(20):
        result = recv(results)
        print("Result:", result)

main()
```

---

## Performance Tips

### Memory Allocation

```python
# Fast: Pre-allocate lists
data = [0] * 1000000  # Single allocation

# Slow: Append in loop
data = []
for i in range(1000000):
    data.append(i)  # Many allocations
```

### Use Arrays for Fixed Sizes

```python
# Fast: Stack-allocated array
nums: [i64; 1000] = [0; 1000]

# Slower: Heap-allocated list
nums = [0] * 1000
```

### Escape Analysis

The compiler automatically optimizes:
- Stack allocation for non-escaping local variables
- Register allocation for simple values
- Dead code elimination

### Optimization Levels

```bash
# No optimization (fast compile)
viper run program.vp

# Basic optimization
viper build program.vp -O 1 -o program

# Default optimization
viper build program.vp -O 2 -o program

# Aggressive optimization
viper build program.vp -O 3 -o program
```

---

## Contributing

See [AGENTS.md](../AGENTS.md) for development guidelines.

## License

MIT License - See LICENSE file for details.
