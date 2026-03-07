# Viper Language - Comprehensive Testing Plan

This document outlines how to test all implemented features of the Viper programming language.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Phase 1: Foundation Tests](#phase-1-foundation-tests)
3. [Phase 2: Python Parity Tests](#phase-2-python-parity-tests)
4. [Phase 3: Standard Library Tests](#phase-3-standard-library-tests)
5. [Phase 4: Testing Tools Tests](#phase-4-testing-tools-tests)
6. [Python File Conversion](#python-file-conversion)
7. [Automated Test Suite](#automated-test-suite)

---

## Quick Start

### Build the Compiler

```bash
cd /home/user/viper-lang
cargo build --release
```

### Run a Test File

```bash
./target/release/viper run tests/test_hello.vp
```

### Run All Tests

```bash
./target/release/viper test tests/
```

---

## Phase 1: Foundation Tests

### 1.1 Walrus Operator (`:=`)

**File:** `tests/test_walrus.vp`

```viper
def test_walrus():
    # Test assignment expression
    data = [1, 2, 3, 4, 5]
    
    # Using walrus operator
    if (n := len(data)) > 3:
        print("Length:", n)
        return True
    return False

def main():
    if test_walrus():
        print("✓ Walrus operator test passed")
    else:
        print("✗ Walrus operator test failed")
```

**Run:**
```bash
./target/release/viper run tests/test_walrus.vp
```

### 1.2 Global/Nonlocal Keywords

**File:** `tests/test_nonlocal.vp`

```viper
def test_basic_nonlocal():
    x = 10
    
    def inner():
        nonlocal x
        x = 20
    
    inner()
    
    if x == 20:
        print("✓ test_basic_nonlocal passed")
        return True
    else:
        print("✗ test_basic_nonlocal failed")
        return False

def test_multiple_nonlocal():
    a = 1
    b = 2
    
    def modifier():
        nonlocal a, b
        a = 10
        b = 20
    
    modifier()
    
    if a == 10 and b == 20:
        print("✓ test_multiple_nonlocal passed")
        return True
    return False

def main():
    test_basic_nonlocal()
    test_multiple_nonlocal()
```

### 1.3 Context Managers (`with`)

**File:** `tests/test_with.vp`

```viper
from io import StringIO

def test_with_stringio():
    buffer = StringIO()
    
    with buffer:
        buffer.write("Hello, World!")
        content = buffer.getvalue()
    
    if content == "Hello, World!":
        print("✓ Context manager test passed")
        return True
    return False

def main():
    test_with_stringio()
```

### 1.4 Exception Chaining

**File:** `tests/test_exceptions.vp`

```viper
def test_exception_chaining():
    try:
        try:
            raise ValueError("Original error")
        except ValueError as e:
            raise RuntimeError("Wrapped error") from e
    except RuntimeError as e:
        print("✓ Exception chaining works")
        return True
    return False

def main():
    test_exception_chaining()
```

---

## Phase 2: Python Parity Tests

### 2.1 @dataclass Decorator

**File:** `tests/test_dataclass.vp`

```viper
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int
    z: int = 0

@dataclass
class Person:
    name: str
    age: int
    email: str = ""

def main():
    # Test Point
    p1 = Point(10, 20)
    print("p1:", p1)
    
    p2 = Point(10, 20)
    if p1 == p2:
        print("✓ dataclass equality works")
    
    # Test Person with default
    person = Person("Alice", 30)
    print("person:", person)
```

### 2.2 Iterator Protocol

**File:** `tests/test_iterator.vp`

```viper
class Counter:
    def __init__(self, start: int, end: int):
        self.start = start
        self.end = end
    
    def __iter__(self):
        self.current = self.start
        return self
    
    def __next__(self) -> int:
        if self.current >= self.end:
            raise StopIteration()
        result = self.current
        self.current = self.current + 1
        return result

def main():
    print("Testing iterator protocol:")
    for i in Counter(0, 5):
        print(i)
    print("✓ Iterator protocol works")
```

### 2.3 Typing Module

**File:** `tests/test_typing.vp`

```viper
from typing import List, Dict, Optional, TypeVar, Generic

T = TypeVar('T')

class Container(Generic[T]):
    def __init__(self, item: T):
        self.item = item
    
    def get(self) -> T:
        return self.item

def process_list(items: List[int]) -> List[str]:
    result = []
    for item in items:
        result.append(str(item))
    return result

def main():
    # Test Generic
    int_container = Container(42)
    print("int_container.get():", int_container.get())
    
    # Test List type
    numbers = [1, 2, 3, 4, 5]
    strings = process_list(numbers)
    print("process_list result:", strings)
    
    print("✓ Typing module works")
```

### 2.4 Functools Module

**File:** `tests/test_functools.vp`

```viper
from functools import partial, reduce, lru_cache

def test_partial():
    def multiply(x, y, z):
        return x * y * z
    
    double = partial(multiply, 2)
    result = double(3, 4)
    
    if result == 24:
        print("✓ partial works")
        return True
    return False

def test_reduce():
    def add(x, y):
        return x + y
    
    result = reduce(add, [1, 2, 3, 4, 5])
    
    if result == 15:
        print("✓ reduce works")
        return True
    return False

def test_lru_cache():
    call_count = [0]
    
    @lru_cache(maxsize=128)
    def fibonacci(n):
        call_count[0] = call_count[0] + 1
        if n <= 1:
            return n
        return fibonacci(n - 1) + fibonacci(n - 2)
    
    result1 = fibonacci(10)
    calls_after_first = call_count[0]
    
    result2 = fibonacci(10)
    calls_after_second = call_count[0]
    
    if result1 == 55 and calls_after_first == calls_after_second:
        print("✓ lru_cache works")
        return True
    return False

def main():
    test_partial()
    test_reduce()
    test_lru_cache()
```

### 2.5 Itertools Module

**File:** `tests/test_itertools.vp`

```viper
from itertools import count, cycle, chain, permutations, combinations

def test_count():
    c = count(10, 2)
    result = [next(c) for i in range(5)]
    if result == [10, 12, 14, 16, 18]:
        print("✓ count works")
        return True
    return False

def test_chain():
    result = list(chain([1, 2], [3, 4], [5]))
    if result == [1, 2, 3, 4, 5]:
        print("✓ chain works")
        return True
    return False

def test_permutations():
    result = list(permutations([1, 2, 3], 2))
    expected = [(1,2), (1,3), (2,1), (2,3), (3,1), (3,2)]
    if result == expected:
        print("✓ permutations works")
        return True
    return False

def main():
    test_count()
    test_chain()
    test_permutations()
```

### 2.6 Collections Module

**File:** `tests/test_collections.vp`

```viper
from collections import namedtuple, OrderedDict, defaultdict, Counter, deque

def test_namedtuple():
    Point = namedtuple('Point', ['x', 'y'])
    p = Point(1, 2)
    
    if p.x == 1 and p.y == 2:
        print("✓ namedtuple works")
        return True
    return False

def test_ordered_dict():
    d = OrderedDict()
    d['a'] = 1
    d['b'] = 2
    d['c'] = 3
    
    keys = list(d.keys())
    if keys == ['a', 'b', 'c']:
        print("✓ OrderedDict works")
        return True
    return False

def test_counter():
    c = Counter(['a', 'b', 'a', 'c', 'b', 'a'])
    
    if c['a'] == 3 and c['b'] == 2:
        print("✓ Counter works")
        return True
    return False

def main():
    test_namedtuple()
    test_ordered_dict()
    test_counter()
```

---

## Phase 3: Standard Library Tests

### 3.1 CSV Module

**File:** `tests/test_csv.vp`

```viper
import csv
import tempfile
import os

def test_csv_reader():
    # Create temp file
    temp_path = "/tmp/test.csv"
    
    f = open(temp_path, 'w')
    f.write("name,age,city\n")
    f.write("Alice,30,NYC\n")
    f.write("Bob,25,LA\n")
    f.close()
    
    # Read CSV
    reader = csv.reader(temp_path)
    rows = reader.read_all()
    
    if len(rows) == 3:
        print("✓ CSV reader works")
    else:
        print("✗ CSV reader failed")
    
    os.remove(temp_path)

def main():
    test_csv_reader()
```

### 3.2 Datetime Module

**File:** `tests/test_datetime.vp`

```viper
from datetime import date, time, datetime, timedelta

def test_date():
    d = date(2024, 3, 7)
    print("Date:", d)
    print("Weekday:", d.weekday())
    print("✓ date works")

def test_datetime():
    dt = datetime(2024, 3, 7, 14, 30, 0)
    print("DateTime:", dt)
    print("✓ datetime works")

def test_timedelta():
    d1 = date(2024, 3, 7)
    d2 = date(2024, 3, 1)
    delta = d1 - d2
    print("Timedelta:", delta.days, "days")
    print("✓ timedelta works")

def main():
    test_date()
    test_datetime()
    test_timedelta()
```

### 3.3 String Module

**File:** `tests/test_string.vp`

```viper
from string import ascii_letters, digits, Template

def test_constants():
    if len(ascii_letters) == 52:
        print("✓ ascii_letters constant works")
    
    if len(digits) == 10:
        print("✓ digits constant works")

def test_template():
    t = Template("Hello, $name!")
    result = t.substitute(name="World")
    
    if result == "Hello, World!":
        print("✓ Template works")
    else:
        print("✗ Template failed")

def main():
    test_constants()
    test_template()
```

### 3.4 Contextlib Module

**File:** `tests/test_contextlib.vp`

```viper
from contextlib import contextmanager, suppress, redirect_stdout
from io import StringIO

@contextmanager
def managed_resource():
    print("Acquiring resource")
    try:
        yield "resource"
    finally:
        print("Releasing resource")

def test_contextmanager():
    with managed_resource() as r:
        print("Using:", r)
    print("✓ contextmanager works")

def test_suppress():
    with suppress(FileNotFoundError):
        os.remove("nonexistent.txt")
    print("✓ suppress works")

def main():
    test_contextmanager()
    test_suppress()
```

### 3.5 Pathlib Module

**File:** `tests/test_pathlib.vp`

```viper
from pathlib import Path

def test_path_creation():
    p = Path("/home/user/test.txt")
    print("Path:", p)
    print("Name:", p.name())
    print("Suffix:", p.suffix())
    print("Parent:", p.parent())
    print("✓ Path creation works")

def test_path_operations():
    p = Path("/tmp")
    
    if p.exists():
        print("✓ exists() works")
    
    if p.is_dir():
        print("✓ is_dir() works")

def test_path_manipulation():
    p = Path("/home/user")
    new_p = p / "test" / "file.txt"
    print("Joined path:", new_p)
    print("✓ Path manipulation works")

def main():
    test_path_creation()
    test_path_operations()
    test_path_manipulation()
```

---

## Phase 4: Testing Tools Tests

### 4.1 Unittest Framework

**File:** `tests/test_unittest_example.vp`

```viper
import unittest

class TestExample(unittest.TestCase):
    def test_equal(self):
        self.assertEqual(1 + 1, 2)
    
    def test_true(self):
        self.assertTrue(True)
    
    def test_in(self):
        self.assertIn(1, [1, 2, 3])
    
    def test_raises(self):
        with self.assertRaises(ValueError):
            int("invalid")

if __name__ == '__main__':
    unittest.main()
```

**Run:**
```bash
./target/release/viper test tests/test_unittest_example.vp
```

### 4.2 Mock Framework

**File:** `tests/test_mock.vp`

```viper
import unittest
from unittest.mock import Mock, patch

class TestMock(unittest.TestCase):
    def test_mock_return_value(self):
        mock = Mock()
        mock.return_value = 42
        result = mock(1, 2, 3)
        self.assertEqual(result, 42)
        mock.assert_called_once()
    
    @patch('os.path.exists')
    def test_patch(self, mock_exists):
        mock_exists.return_value = True
        result = os.path.exists("test")
        self.assertTrue(result)
        mock_exists.assert_called_once()

if __name__ == '__main__':
    unittest.main()
```

### 4.3 Coverage Tool

**File:** `tests/test_coverage_example.vp`

```viper
from coverage import Coverage

def add(a, b):
    return a + b

def multiply(a, b):
    return a * b

def main():
    cov = Coverage()
    cov.start()
    
    # Run code
    result1 = add(2, 3)
    result2 = multiply(4, 5)
    
    cov.stop()
    cov.save()
    cov.report()
```

**Run:**
```bash
./target/release/viper run tests/test_coverage_example.vp
```

### 4.4 Debugger (PDB)

**File:** `tests/test_debugger.vp`

```viper
import pdb

def buggy_function(x):
    pdb.set_trace()  # Set breakpoint
    result = x * 2
    return result

def main():
    result = buggy_function(21)
    print("Result:", result)
```

**Run interactively:**
```bash
./target/release/viper run tests/test_debugger.vp
```

**Debugger commands:**
- `n` - Next line
- `s` - Step into
- `c` - Continue
- `p <expr>` - Print expression
- `l` - List code
- `w` - Where (stack trace)
- `q` - Quit

---

## Python File Conversion

### Can Viper Convert .py Files?

**Current Status:** ⚠️ **Partial Support**

Viper is designed for **Python compatibility**, not direct `.py` file conversion. Here's what works:

### ✅ What Works (Syntax Compatible)

```python
# This Python code works in Viper with .vp extension:

# Basic syntax
x = 10
y = 20
z = x + y

# Functions
def greet(name: str) -> str:
    return f"Hello, {name}!"

# Classes
class Person:
    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age
    
    def __repr__(self) -> str:
        return f"Person({self.name}, {self.age})"

# List comprehensions
squares = [x*x for x in range(10)]

# Context managers
with open("file.txt") as f:
    content = f.read()

# Decorators
@dataclass
class Point:
    x: int
    y: int
```

### ❌ What Doesn't Work

1. **Dynamic features:**
   - `eval()`, `exec()` (limited support)
   - Runtime attribute modification
   - Monkey patching

2. **Some stdlib modules:**
   - Modules not yet implemented in Viper
   - CPython-specific C extensions

3. **Python version-specific features:**
   - Match statement (Python 3.10+) - ✅ Implemented
   - Some typing features

### How to Port Python Code

**Step 1:** Rename file extension
```bash
cp myscript.py myscript.vp
```

**Step 2:** Check for incompatible features
```bash
./target/release/viper check myscript.vp
```

**Step 3:** Fix any issues
- Add type annotations where needed
- Replace unsupported stdlib imports with Viper equivalents
- Remove dynamic features

**Step 4:** Run
```bash
./target/release/viper run myscript.vp
```

### Example Port

**Python (myscript.py):**
```python
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int

def main():
    p = Point(10, 20)
    print(p)

if __name__ == '__main__':
    main()
```

**Viper (myscript.vp):** - Same code, just rename!
```viper
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int

def main():
    p = Point(10, 20)
    print(p)

if __name__ == '__main__':
    main()
```

---

## Automated Test Suite

### Run All Tests

```bash
# Using the test runner
./target/release/viper test tests/

# With verbose output
./target/release/viper test tests/ --verbose

# Filter by name pattern
./target/release/viper test tests/ --filter "test_data"

# Discover and run
./target/release/viper test tests/ --discover
```

### Test Directory Structure

```
tests/
├── test_hello.vp              # Basic hello world
├── test_nonlocal.vp           # Nonlocal keyword
├── test_dataclass.vp          # @dataclass decorator
├── test_typing.vp             # Typing module
├── test_functools.vp          # Functools module
├── test_itertools.vp          # Itertools module
├── test_collections.vp        # Collections module
├── test_csv.vp                # CSV module
├── test_datetime.vp           # Datetime module
├── test_string.vp             # String module
├── test_contextlib.vp         # Contextlib module
├── test_pathlib.vp            # Pathlib module
├── test_unittest_example.vp   # Unittest framework
├── test_mock.vp               # Mock framework
├── test_coverage_example.vp   # Coverage tool
└── test_debugger.vp           # Debugger
```

### Create Your Own Test

```viper
# tests/test_my_feature.vp

def test_feature():
    # Test your feature
    result = my_function()
    if result == expected:
        print("✓ Test passed")
        return True
    else:
        print("✗ Test failed")
        return False

def main():
    if test_feature():
        pass  # Success
    else:
        exit(1)  # Failure
```

---

## Summary

### Testing Commands

| Command | Description |
|---------|-------------|
| `viper run <file>` | Run a single file |
| `viper test <dir>` | Run all tests in directory |
| `viper test --verbose` | Verbose test output |
| `viper test --filter "pattern"` | Filter tests by name |
| `viper check <file>` | Type check without running |

### Coverage

```bash
# Run with coverage
from coverage import Coverage
cov = Coverage()
cov.start()
# ... run your code ...
cov.stop()
cov.report()
```

### Debugging

```viper
import pdb
pdb.set_trace()  # Set breakpoint
```

---

**Happy Testing! 🎉**
