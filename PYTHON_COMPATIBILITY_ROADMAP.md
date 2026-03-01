# Viper Language: Python Compatibility Roadmap

## Executive Summary

**Current State:** ~85% syntax compatibility, ~10% stdlib coverage, basic testing  
**Target State:** ~98% syntax compatibility, ~80% stdlib coverage, production-ready testing  
**Timeline:** 18-24 months with 2-3 dedicated developers

---

## Phase 1: Foundation Completion (Months 1-3)

### Goal: Complete all Phase 2-3 language features and core stdlib

### 1.1 Language Features

| Feature | Effort | Priority | Dependencies |
|---------|--------|----------|--------------|
| Walrus operator (`:=`) | 2 days | High | Parser, semantic analysis |
| `global` / `nonlocal` | 3 days | High | Symbol table, codegen |
| Loop `else` clauses | 2 days | Medium | Parser, codegen |
| Multiple inheritance (C3 MRO) | 5 days | Medium | OOP system |
| Context managers (`with`) | 4 days | High | AST, codegen, runtime |
| Exception chaining (`raise from`) | 3 days | Medium | Exception system |

**Implementation Steps:**
```
1. Add AST nodes for new features
2. Extend parser (recursive descent)
3. Add semantic analysis rules
4. Generate LLVM IR
5. Write integration tests
```

### 1.2 Standard Library Completion

| Module | C Runtime | Viper Wrapper | JIT Stubs | Effort |
|--------|-----------|---------------|-----------|--------|
| `math` | ✅ | ✅ | ⏳ | 1 day |
| `json` | ✅ | ✅ | ⏳ | 1 day |
| `collections` | ✅ | ✅ | ⏳ | 2 days |
| `re` | ✅ | ✅ | ⏳ | 2 days |
| `random` | ✅ | ✅ | ⏳ | 1 day |
| `socket` | ✅ | ⏳ | ⏳ | 3 days |
| `asyncio` | ✅ | ⏳ | ⏳ | 5 days |
| `http` | ✅ | ⏳ | ⏳ | 4 days |
| `select` | ✅ | ⏳ | ⏳ | 2 days |
| `hashlib` | ✅ | ⏳ | ⏳ | 2 days |
| `decimal` | ✅ | ⏳ | ⏳ | 3 days |
| `logging` | ✅ | ⏳ | ⏳ | 2 days |

**Total: 28 days for stdlib completion**

### 1.3 Testing Infrastructure

```rust
// tests/framework/mod.rs - New test runner
pub struct TestRunner {
    tests: Vec<TestCase>,
    passed: usize,
    failed: usize,
}

pub struct TestCase {
    name: String,
    source: String,
    expected_output: String,
    expected_exit_code: i32,
}

// Usage in test files
#[test]
fn test_list_comprehension() {
    run_test("tests/test_list_comp.vp", expected_output);
}
```

**Deliverables:**
- ✅ All Phase 2-3 language features
- ✅ 12 core stdlib modules fully wired
- ✅ Rust-based test runner (`viper test`)
- ✅ 100+ integration tests

---

## Phase 2: Python Parity - Syntax & Semantics (Months 4-7)

### Goal: Achieve 95% Python syntax compatibility

### 2.1 Advanced Language Features

| Feature | Effort | Priority | Python Equivalent |
|---------|--------|----------|-------------------|
| Union types (`int | str`) | 4 days | High | `typing.Union` |
| Generic types (`List[T]`) | 5 days | High | `typing.Generic` |
| Function types (`fn(int) -> str`) | 3 days | Medium | `typing.Callable` |
| Type aliases | 2 days | Medium | `type Alias = ...` |
| Named tuples | 3 days | Medium | `collections.namedtuple` |
| `@dataclass` decorator | 4 days | High | `dataclasses.dataclass` |
| `@staticmethod`, `@classmethod` | 2 days | High | Same |
| `@property` decorator | 3 days | High | Same |
| `__enter__` / `__exit__` | 3 days | High | Context managers |
| `__iter__` / `__next__` | 2 days | High | Iterators |
| `__getitem__` / `__setitem__` | 2 days | High | Subscriptable |
| `__call__` | 1 day | Medium | Callable objects |
| `__str__` / `__repr__` | 1 day | High | String representation |
| `__eq__` / `__lt__` / etc. | 2 days | High | Comparisons |
| `__add__` / `__mul__` / etc. | 3 days | Medium | Operator overloading |
| `async for` | 4 days | Medium | Async iteration |
| `async with` | 3 days | Medium | Async context |
| `await` in comprehensions | 2 days | Low | Async comprehensions |

**Implementation Strategy:**
```
Week 1-2: Type system extensions (union, generic, function types)
Week 3-4: Decorator system overhaul
Week 5-6: Special methods (dunder methods)
Week 7-8: Async enhancements
Week 9-10: Integration testing + bug fixes
```

### 2.2 Type System Overhaul

```viper
# Before (current)
def process(data):  # Type inferred
    return data

# After (Phase 2)
from typing import Generic, TypeVar, Callable, Optional, Union

T = TypeVar('T')
U = TypeVar('U')

def process(data: list[int]) -> list[str]:
    return [str(x) for x in data]

def compose(f: fn(int) -> str, g: fn(str) -> bool) -> fn(int) -> bool:
    return lambda x: g(f(x))

class Container(Generic[T]):
    def __init__(self, item: T):
        self.item = item
    
    def get(self) -> T:
        return self.item

MaybeInt = Optional[int]  # Type alias
Result = Union[int, str]  # Union type
```

### 2.3 Compatibility Layer

```viper
# std/compat/python_syntax.vp
# Allow Python developers to write familiar code

# Python-style print (variadic)
def print(*args, sep=" ", end="\n"):
    # Implementation using existing vp_print_*

# Python-style range with step
def range(start, stop=None, step=1):
    # Implementation

# Python-style list methods
class list:
    def extend(self, other): ...
    def index(self, item, start=0, end=None): ...
    def count(self, item): ...
    def sort(self, key=None, reverse=False): ...
```

**Deliverables:**
- ✅ 95% Python syntax compatibility
- ✅ Full type system (generics, unions, callables)
- ✅ Complete decorator support
- ✅ All special methods implemented
- ✅ Python compatibility module (`std/compat`)

---

## Phase 3: Standard Library Expansion (Months 8-12)

### Goal: Implement 50+ Python-compatible stdlib modules

### 3.1 Module Implementation Priority

#### Tier 1: Essential (Month 8-9)

| Module | Functions | Effort | Python Docs |
|--------|-----------|--------|-------------|
| `builtins` | 30+ builtins | 5 days | Python builtins |
| `typing` | Full typing support | 5 days | typing module |
| `io` | StringIO, BytesIO, open | 4 days | io module |
| `pathlib` | Path class (complete) | 3 days | pathlib module |
| `copy` | copy, deepcopy | 2 days | copy module |
| `functools` | partial, reduce, lru_cache | 3 days | functools module |
| `itertools` | 20+ iterators | 4 days | itertools module |
| `operator` | itemgetter, attrgetter | 2 days | operator module |

#### Tier 2: Data & Serialization (Month 10)

| Module | Functions | Effort | Python Docs |
|--------|-----------|--------|-------------|
| `csv` | reader, writer, DictReader | 3 days | csv module |
| `pickle` | dump, load, dumps, loads | 5 days | pickle module |
| `base64` | encode, decode | 2 days | base64 module |
| `configparser` | ConfigParser class | 3 days | configparser module |
| `dataclasses` | dataclass, field | 4 days | dataclasses module |
| `enum` | Enum, IntEnum, Flag | 3 days | enum module |
| `warnings` | warn, filterwarnings | 2 days | warnings module |

#### Tier 3: System & I/O (Month 11)

| Module | Functions | Effort | Python Docs |
|--------|-----------|--------|-------------|
| `shutil` | copy, move, rmtree | 3 days | shutil module |
| `glob` | glob, iglob, escape | 2 days | glob module |
| `tempfile` | NamedTemporaryFile, mkdtemp | 3 days | tempfile module |
| `fileinput` | FileInput class | 2 days | fileinput module |
| `mmap` | mmap class | 4 days | mmap module |
| `subprocess` | run, Popen, call | 5 days | subprocess module |
| `signal` | signal, SIG_IGN, SIG_DFL | 3 days | signal module |

#### Tier 4: Text & Encoding (Month 12)

| Module | Functions | Effort | Python Docs |
|--------|-----------|--------|-------------|
| `string` | Template, constants | 2 days | string module |
| `textwrap` | wrap, fill, dedent | 2 days | textwrap module |
| `unicodedata` | lookup, name, category | 3 days | unicodedata module |
| `codecs` | encode, decode, register | 3 days | codecs module |
| `locale` | locale, formatting | 3 days | locale module |
| `gettext` | gettext, ngettext | 3 days | gettext module |

### 3.2 Implementation Pattern

```c
// runtime/src/csv.c - Example C runtime
#include "viper_stdlib.h"

typedef struct {
    ViperArc* fields;  // Array of strings
    i64 field_count;
} ViperCsvRow;

ViperCsvRow* vp_csv_row_create(i64 field_count) {
    // Implementation
}

void vp_csv_row_free(ViperCsvRow* row) {
    // Implementation
}

ViperArc* vp_csv_read_file(const char* filename) {
    // Parse CSV file, return list of rows
}

char* vp_csv_write_rows(ViperArc* rows) {
    // Serialize to CSV string
}
```

```viper
# std/csv.vp - Viper wrapper
from typing import List, Dict, Optional

class CsvReader:
    def __init__(self, filepath: str, delimiter: str = ","):
        self.filepath = filepath
        self.delimiter = delimiter
    
    def __iter__(self) -> Iterator[list[str]]:
        # Yield rows
    
    def read_all(self) -> list[list[str]]:
        # Return all rows

class DictReader:
    def __init__(self, filepath: str, fieldnames: list[str] = None):
        self.filepath = filepath
        self.fieldnames = fieldnames
    
    def __iter__(self) -> Iterator[Dict[str, str]]:
        # Yield dicts
```

### 3.3 Testing Strategy

```viper
# tests/test_stdlib/test_csv.vp
import csv
import tempfile
import os

def test_csv_reader():
    # Create temp file
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write("name,age,city\n")
        f.write("Alice,30,NYC\n")
        f.write("Bob,25,LA\n")
        temp_path = f.name
    
    try:
        reader = csv.reader(temp_path)
        rows = reader.read_all()
        
        assert len(rows) == 3
        assert rows[0] == ["name", "age", "city"]
        assert rows[1][0] == "Alice"
        
        print("✓ test_csv_reader passed")
    finally:
        os.remove(temp_path)

def test_dict_reader():
    # Similar test for DictReader
    pass

# Run all stdlib tests
def main():
    test_csv_reader()
    test_dict_reader()
    # ... more tests
```

**Deliverables:**
- ✅ 50+ stdlib modules
- ✅ 500+ stdlib functions
- ✅ 300+ integration tests
- ✅ Python stdlib compatibility guide

---

## Phase 4: Testing & Tooling Ecosystem (Months 13-17)

### Goal: Production-ready testing, debugging, and developer tools

### 4.1 Unit Testing Framework

```viper
# std/unittest.vp - Python-compatible unittest
from typing import Callable, Optional

class TestCase:
    def __init__(self, name: str):
        self.name = name
        self._passed = False
        self._failure_message: Optional[str] = None
    
    def setUp(self):
        pass
    
    def tearDown(self):
        pass
    
    def run(self):
        self.setUp()
        try:
            self.test_method()
            self._passed = True
        except AssertionError as e:
            self._failure_message = str(e)
        finally:
            self.tearDown()
    
    def assertEqual(self, a, b, msg: str = None):
        if a != b:
            raise AssertionError(msg or f"{a!r} != {b!r}")
    
    def assertTrue(self, expr, msg: str = None):
        if not expr:
            raise AssertionError(msg or f"{expr!r} is not True")
    
    def assertFalse(self, expr, msg: str = None):
        if expr:
            raise AssertionError(msg or f"{expr!r} is not False")
    
    def assertRaises(self, exc_type: type, callable: Callable = None):
        # Context manager or callable wrapper
        pass
    
    def assertAlmostEqual(self, a, b, places: int = 7):
        # Float comparison
        pass
    
    def assertIn(self, item, container):
        if item not in container:
            raise AssertionError(f"{item!r} not in {container!r}")
    
    def assertIsNone(self, obj):
        if obj is not None:
            raise AssertionError(f"{obj!r} is not None")
    
    def assertIsNotNone(self, obj):
        if obj is None:
            raise AssertionError(f"{obj!r} is None")

class TestSuite:
    def __init__(self):
        self.tests: list[TestCase] = []
    
    def addTest(self, test: TestCase):
        self.tests.append(test)
    
    def run(self) -> TestResult:
        result = TestResult()
        for test in self.tests:
            test.run()
            if test._passed:
                result.addSuccess(test)
            else:
                result.addFailure(test, test._failure_message)
        return result

class TestResult:
    def __init__(self):
        self.testsRun = 0
        self.failures: list[tuple[TestCase, str]] = []
        self.errors: list[tuple[TestCase, str]] = []
        self.skipped: list[tuple[TestCase, str]] = []
    
    def addSuccess(self, test):
        self.testsRun += 1
    
    def addFailure(self, test, err):
        self.testsRun += 1
        self.failures.append((test, err))
    
    def wasSuccessful(self) -> bool:
        return len(self.failures) == 0 and len(self.errors) == 0

def main():
    suite = TestSuite()
    suite.addTest(MyTestCase("test_something"))
    result = suite.run()
    
    print(f"Ran {result.testsRun} tests")
    if result.wasSuccessful():
        print("OK")
    else:
        for test, err in result.failures:
            print(f"FAIL: {test.name}")
            print(f"  {err}")
```

### 4.2 Test Discovery & Runner

```viper
# std/unittest/runner.vp
import os
import sys
import importlib

class TextTestRunner:
    def __init__(self, verbosity: int = 1):
        self.verbosity = verbosity
    
    def run(self, test) -> TestResult:
        # Run tests with text output
        pass

def defaultTestLoader(discover_dir: str = ".", pattern: str = "test_*.vp"):
    """Discover and load tests from directory."""
    tests = []
    for root, dirs, files in os.walk(discover_dir):
        for filename in files:
            if filename.startswith(pattern[:-1]) and filename.endswith(".vp"):
                filepath = os.path.join(root, filename)
                module = importlib.import_module(filepath)
                # Find TestCase classes
                for name in dir(module):
                    obj = getattr(module, name)
                    if isinstance(obj, type) and issubclass(obj, TestCase):
                        tests.append(obj())
    return tests

def main(module=None):
    """CLI entry point: viper -m unittest"""
    if module is None:
        tests = defaultTestLoader()
    else:
        tests = [module]
    
    runner = TextTestRunner(verbosity=2)
    result = runner.run(tests)
    
    sys.exit(0 if result.wasSuccessful() else 1)
```

### 4.3 Mocking Framework

```viper
# std/unittest/mock.vp
from typing import Any, Callable, Optional

class Mock:
    def __init__(self, spec=None, return_value=None, side_effect=None):
        self.spec = spec
        self.return_value = return_value
        self.side_effect = side_effect
        self.call_count = 0
        self.call_args_list: list[tuple] = []
        self._children: dict[str, Mock] = {}
    
    def __call__(self, *args, **kwargs):
        self.call_count += 1
        self.call_args_list.append((args, kwargs))
        
        if self.side_effect is not None:
            if callable(self.side_effect):
                return self.side_effect(*args, **kwargs)
            elif isinstance(self.side_effect, Exception):
                raise self.side_effect
            elif hasattr(self.side_effect, '__iter__'):
                return next(iter(self.side_effect))
        
        return self.return_value
    
    def __getattr__(self, name: str) -> Mock:
        if name not in self._children:
            self._children[name] = Mock()
        return self._children[name]
    
    def assert_called(self):
        if self.call_count == 0:
            raise AssertionError("Expected mock to have been called")
    
    def assert_called_once(self):
        if self.call_count != 1:
            raise AssertionError(f"Expected 1 call, got {self.call_count}")
    
    def assert_called_with(self, *args, **kwargs):
        if (args, kwargs) not in self.call_args_list:
            raise AssertionError(f"Expected call {args, kwargs} not found")
    
    def reset_mock(self):
        self.call_count = 0
        self.call_args_list.clear()
        for child in self._children.values():
            child.reset_mock()

class patch:
    """Context manager for mocking."""
    def __init__(self, target: str, new=None, new_callable=None):
        self.target = target
        self.new = new
        self.new_callable = new_callable
        self._original = None
    
    def __enter__(self):
        # Save original, inject mock
        self._original = getattr(self.target)
        mock = self.new or Mock()
        setattr(self.target, mock)
        return mock
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        # Restore original
        setattr(self.target, self._original)
```

### 4.4 Debugger Integration

```viper
# std/pdb.vp - Python-compatible debugger
import sys
import readline

class Pdb:
    def __init__(self):
        self.prompt = "(Pdb) "
        self._current_frame = None
        self._breakpoints: list[Breakpoint] = []
    
    def set_trace(self, frame=None):
        """Start debugging at current frame."""
        self._current_frame = frame or sys._getframe(1)
        self.cmdloop()
    
    def cmdloop(self):
        """Read-eval-print loop for debugger commands."""
        while True:
            try:
                line = input(self.prompt)
                if not self.onecmd(line):
                    break
            except EOFError:
                break
    
    def onecmd(self, line: str) -> bool:
        """Execute single command. Returns True to exit."""
        parts = line.split()
        if not parts:
            return False
        
        cmd = parts[0]
        args = parts[1:]
        
        if cmd in ('q', 'quit', 'exit'):
            return True
        elif cmd in ('n', 'next'):
            self.do_next()
        elif cmd in ('s', 'step'):
            self.do_step()
        elif cmd in ('c', 'continue'):
            self.do_continue()
        elif cmd in ('b', 'break'):
            self.do_break(args)
        elif cmd in ('p', 'print'):
            self.do_print(args)
        elif cmd in ('l', 'list'):
            self.do_list()
        elif cmd in ('w', 'where'):
            self.do_where()
        elif cmd in ('h', 'help'):
            self.do_help()
        else:
            print(f"Unknown command: {cmd}")
        
        return False
    
    def do_next(self):
        """Execute current line, step over function calls."""
        # Implementation using debug hooks
    
    def do_step(self):
        """Step into function calls."""
        pass
    
    def do_continue(self):
        """Continue execution until next breakpoint."""
        pass
    
    def do_break(self, args):
        """Set breakpoint at line number or function."""
        if not args:
            print("Usage: break <line|function>")
            return
        
        location = args[0]
        # Parse location, add breakpoint
        self._breakpoints.append(Breakpoint(location))
        print(f"Breakpoint at {location}")
    
    def do_print(self, args):
        """Print variable value."""
        expr = ' '.join(args)
        # Evaluate in current frame context
        value = eval(expr, self._current_frame.f_locals)
        print(repr(value))
    
    def do_list(self):
        """List source code around current line."""
        pass
    
    def do_where(self):
        """Print stack trace."""
        frame = self._current_frame
        while frame is not None:
            filename = frame.f_code.co_filename
            lineno = frame.f_lineno
            funcname = frame.f_code.co_name
            print(f"  {filename}:{lineno} in {funcname}")
            frame = frame.f_back

# Global debugger instance
pdb = Pdb()

def set_trace():
    """Start debugging."""
    pdb.set_trace()

def breakpoint():
    """Built-in breakpoint() calls this."""
    pdb.set_trace()
```

### 4.5 Code Coverage

```viper
# std/coverage.vp
import sys
import os

class Coverage:
    def __init__(self):
        self._data: dict[str, set[int]] = {}  # file -> lines executed
        self._enabled = False
    
    def start(self):
        """Start collecting coverage data."""
        self._enabled = True
        sys.settrace(self._trace_function)
    
    def stop(self):
        """Stop collecting coverage data."""
        self._enabled = False
        sys.settrace(None)
    
    def _trace_function(self, frame, event, arg):
        if event == 'line':
            filename = frame.f_code.co_filename
            lineno = frame.f_lineno
            
            if filename not in self._data:
                self._data[filename] = set()
            self._data[filename].add(lineno)
        
        return self._trace_function
    
    def save(self, filepath: str = ".coverage"):
        """Save coverage data to file."""
        import json
        data = {
            filename: sorted(lines)
            for filename, lines in self._data.items()
        }
        with open(filepath, 'w') as f:
            json.dump(data, f)
    
    def load(self, filepath: str = ".coverage"):
        """Load coverage data from file."""
        import json
        with open(filepath, 'r') as f:
            data = json.load(f)
        self._data = {
            filename: set(lines)
            for filename, lines in data.items()
        }
    
    def report(self) -> str:
        """Generate coverage report."""
        lines = []
        total_files = 0
        total_lines = 0
        covered_lines = 0
        
        for filename, executed in self._data.items():
            # Count total lines in file
            with open(filename, 'r') as f:
                all_lines = len(f.readlines())
            
            covered = len(executed)
            pct = (covered / all_lines * 100) if all_lines > 0 else 0
            
            lines.append(f"{filename}: {covered}/{all_lines} ({pct:.1f}%)")
            
            total_files += 1
            total_lines += all_lines
            covered_lines += covered
        
        overall_pct = (covered_lines / total_lines * 100) if total_lines > 0 else 0
        lines.append(f"\nTOTAL: {covered_lines}/{total_lines} ({overall_pct:.1f}%)")
        
        return '\n'.join(lines)

# CLI usage: viper -m coverage run test.vp
#            viper -m coverage report
```

### 4.6 CLI Tool Enhancements

```rust
// src/cli/test.rs - New test command
pub fn run_test_command(args: TestArgs) -> Result<(), String> {
    let mut runner = TestRunner::new();
    
    if args.discover {
        runner.discover(&args.path)?;
    } else {
        runner.load_file(&args.path)?;
    }
    
    let result = runner.run()?;
    
    // Print results
    println!("Ran {} tests", result.tests_run);
    println!("Passed: {}", result.passed);
    println!("Failed: {}", result.failed);
    
    if !result.failures.is_empty() {
        println!("\nFailures:");
        for failure in &result.failures {
            println!("  - {}", failure.test_name);
            println!("    {}", failure.message);
        }
    }
    
    Ok(())
}
```

**Deliverables:**
- ✅ `unittest` framework (unittest.TestCase compatible)
- ✅ Test discovery and runner
- ✅ Mocking framework (unittest.mock compatible)
- ✅ Debugger (pdb compatible)
- ✅ Code coverage tool
- ✅ `viper test` CLI command
- ✅ 500+ unit tests for stdlib

---

## Phase 5: Advanced Features & Optimization (Months 18-24)

### Goal: Match Python's advanced features and exceed in performance

### 5.1 Metaprogramming

| Feature | Effort | Python Equivalent |
|---------|--------|-------------------|
| Class decorators | 3 days | `@decorator` on classes |
| `__new__` method | 2 days | Object creation |
| `__init_subclass__` | 2 days | Subclass initialization |
| `__class_getitem__` | 2 days | Generic subscription |
| `__annotations__` | 1 day | Type hints storage |
| `inspect` module | 5 days | Runtime introspection |
| `ast` module | 5 days | AST manipulation |
| `dis` module | 3 days | Disassembly |

```viper
# std/inspect.vp
from typing import Any, Optional, List

def getmembers(object, predicate=None) -> list[tuple[str, Any]]:
    """Return all members of an object as (name, value) pairs."""
    members = []
    for name in dir(object):
        value = getattr(object, name)
        if predicate is None or predicate(value):
            members.append((name, value))
    return members

def getsource(object) -> str:
    """Return source code of an object."""
    # Use compiler's AST + source map
    pass

def getfile(object) -> str:
    """Return file path where object was defined."""
    pass

def getmodule(object) -> Optional[object]:
    """Return module an object was defined in."""
    pass

def signature(callable) -> Signature:
    """Get signature of a callable."""
    pass

def isclass(object) -> bool:
    """Return True if object is a class."""
    pass

def isfunction(object) -> bool:
    """Return True if object is a function."""
    pass

def ismethod(object) -> bool:
    """Return True if object is a bound method."""
    pass

def isbuiltin(object) -> bool:
    """Return True if object is a built-in function."""
    pass

def ismodule(object) -> bool:
    """Return True if object is a module."""
    pass
```

### 5.2 Advanced OOP

| Feature | Effort | Python Equivalent |
|---------|--------|-------------------|
| Descriptors | 4 days | `__get__`, `__set__`, `__delete__` |
| Metaclasses | 5 days | `class MyClass(metaclass=Meta)` |
| `__prepare__` | 2 days | Metaclass class prep |
| `__instancecheck__` | 2 days | `isinstance()` override |
| `__subclasscheck__` | 2 days | `issubclass()` override |
| `@abstractmethod` | 2 days | Abstract base classes |
| `ABC` class | 2 days | Abstract base class |
| `@final` decorator | 1 day | Prevent inheritance |
| `@override` decorator | 1 day | Mark method override |
| `__slots__` | 3 days | Memory optimization |

```viper
# std/abc.vp - Abstract Base Classes
from typing import TypeVar, Generic

T = TypeVar('T')

class ABCMeta(type):
    """Metaclass for abstract base classes."""
    def __new__(mcs, name, bases, namespace):
        cls = super().__new__(mcs, name, bases, namespace)
        
        # Collect abstract methods
        abstract_methods = []
        for attr_name, attr_value in namespace.items():
            if hasattr(attr_value, '__is_abstract__'):
                abstract_methods.append(attr_name)
        
        cls.__abstract_methods__ = abstract_methods
        return cls
    
    def __call__(cls, *args, **kwargs):
        # Check for unimplemented abstract methods
        instance = super().__call__(*args, **kwargs)
        
        for method_name in cls.__abstract_methods__:
            method = getattr(instance, method_name)
            if hasattr(method, '__is_abstract__'):
                raise TypeError(
                    f"Can't instantiate abstract class {cls.__name__} "
                    f"with abstract method {method_name}"
                )
        
        return instance

class ABC(metaclass=ABCMeta):
    """Helper base class for abstract base classes."""
    pass

def abstractmethod(func):
    """Decorator to mark a method as abstract."""
    func.__is_abstract__ = True
    return func

# Example usage
class Shape(ABC):
    @abstractmethod
    def area(self) -> float:
        pass
    
    @abstractmethod
    def perimeter(self) -> float:
        pass

class Rectangle(Shape):
    def __init__(self, width: float, height: float):
        self.width = width
        self.height = height
    
    def area(self) -> float:
        return self.width * self.height
    
    def perimeter(self) -> float:
        return 2 * (self.width + self.height)
```

### 5.3 Remaining Stdlib Modules

| Module | Effort | Priority |
|--------|--------|----------|
| `statistics` | 3 days | Medium |
| `fractions` | 2 days | Low |
| `numbers` | 2 days | Low |
| `contextlib` | 3 days | High |
| `weakref` | 3 days | Medium |
| `types` | 2 days | Medium |
| `pprint` | 2 days | Medium |
| `reprlib` | 1 day | Low |
| `datetime` (complete) | 5 days | High |
| `calendar` | 3 days | Low |
| `zoneinfo` | 4 days | Medium |
| `ssl` | 5 days | Medium |
| `selectors` | 3 days | Low |
| `concurrent.futures` | 5 days | Medium |
| `hmac` | 2 days | Low |
| `secrets` | 1 day | Low |
| `sqlite3` | 7 days | Medium |
| `dbm` | 3 days | Low |
| `shelve` | 2 days | Low |

### 5.4 Performance Optimization

| Feature | Effort | Impact |
|---------|--------|--------|
| SIMD auto-vectorization | 10 days | High (numeric code) |
| Profile-guided optimization | 5 days | Medium (10-20% speedup) |
| Link-time optimization | 3 days | Medium (cross-module inlining) |
| Interprocedural analysis | 7 days | High (better optimization) |
| Escape analysis improvements | 5 days | High (more stack allocation) |
| Inline caching | 4 days | Medium (attribute access) |
| Method lookup caching | 3 days | Medium (OOP performance) |
| String interning | 3 days | Medium (memory + speed) |
| Small integer caching | 2 days | Low (micro-optimization) |
| List comprehension optimization | 3 days | High (common pattern) |

### 5.5 Developer Tools

| Tool | Effort | Priority |
|------|--------|----------|
| `viper fmt` (complete) | 10 days | High |
| `viper lint` | 14 days | High |
| `viper doc` | 10 days | Medium |
| `viper bench` | 7 days | Medium |
| `vpm` (package manager) | 21 days | High |
| `viper-lsp` | 28 days | High |
| VS Code extension | 14 days | Medium |
| Vim/Neovim plugin | 7 days | Low |

### 5.6 Documentation

| Document | Effort | Priority |
|----------|--------|----------|
| Language reference | 10 days | High |
| Standard library docs | 14 days | High |
| Tutorial (beginner) | 7 days | High |
| Migration guide (Python → Viper) | 5 days | High |
| API documentation generator | 7 days | Medium |
| Examples cookbook | 7 days | Medium |

**Deliverables:**
- ✅ Full metaprogramming support
- ✅ Complete OOP (descriptors, metaclasses)
- ✅ 80+ stdlib modules total
- ✅ Performance optimizations (SIMD, PGO, LTO)
- ✅ Complete toolchain (fmt, lint, doc, bench)
- ✅ Package manager (`vpm`)
- ✅ Language server (`viper-lsp`)
- ✅ Comprehensive documentation

---

## Resource Requirements

### Team Composition

| Role | Count | Duration |
|------|-------|----------|
| Compiler engineer (Rust/LLVM) | 1-2 | Full 24 months |
| Runtime engineer (C) | 1 | Months 1-12 |
| Stdlib developer (Viper/C) | 1-2 | Months 1-18 |
| Tooling developer | 1 | Months 13-24 |
| Documentation writer | 0.5 | Months 18-24 |

### Infrastructure

| Resource | Cost/Month |
|----------|------------|
| CI/CD (GitHub Actions) | $0-50 |
| Build servers | $100-200 |
| Documentation hosting | $0-20 |
| Package registry | $0-50 |
| **Total** | **$100-320/month** |

---

## Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LLVM version incompatibility | Medium | High | Pin LLVM version, test upgrades early |
| Runtime performance gaps | Medium | Medium | Benchmark early, optimize hot paths |
| Stdlib scope creep | High | Medium | Prioritize by usage, defer nice-to-haves |
| Developer burnout | Medium | High | Modular work, clear milestones |
| Community adoption | Medium | Medium | Python compatibility focus, migration tools |

---

## Success Metrics

| Metric | Current | Phase 3 | Phase 5 Target |
|--------|---------|---------|----------------|
| Syntax compatibility | 85% | 95% | 98% |
| Stdlib coverage | 10% | 40% | 80% |
| Test count | 50 | 300 | 2000+ |
| Build time (O2) | 100ms | 150ms | 200ms |
| Runtime speed vs C | 3-5x | 2-3x | 1.5-2x |
| Documentation coverage | 30% | 60% | 95% |
| Python code portability | 50% | 75% | 90% |

---

## Milestones Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| **Phase 1** | Months 1-3 | Core language features, 12 stdlib modules, test runner |
| **Phase 2** | Months 4-7 | 95% syntax compatibility, type system, decorators |
| **Phase 3** | Months 8-12 | 50+ stdlib modules, Python compatibility layer |
| **Phase 4** | Months 13-17 | unittest, mock, debugger, coverage, tooling |
| **Phase 5** | Months 18-24 | Metaprogramming, optimization, 80+ modules, full toolchain |

---

## Appendix A: Python Compatibility Quick Reference

### Syntax Compatibility Matrix

| Feature | Current | Phase 2 | Phase 5 |
|---------|---------|---------|---------|
| Indentation-based blocks | ✅ | ✅ | ✅ |
| Function definitions | ✅ | ✅ | ✅ |
| Class definitions | ⚠️ | ✅ | ✅ |
| Decorators | ⚠️ | ✅ | ✅ |
| Type hints | ⚠️ | ✅ | ✅ |
| Comprehensions | ⚠️ | ✅ | ✅ |
| Pattern matching | ✅ | ✅ | ✅ |
| Async/await | ⚠️ | ✅ | ✅ |
| Walrus operator | ❌ | ✅ | ✅ |
| `global` keyword | ❌ | ✅ | ✅ |
| `nonlocal` keyword | ❌ | 🔄 | ✅ |
| `@dataclass` | ❌ | ✅ | ✅ |
| `__slots__` | ❌ | ❌ | ✅ |
| Metaclasses | ❌ | ❌ | ✅ |

### Stdlib Coverage by Category

| Category | Current | Phase 3 | Phase 5 |
|----------|---------|---------|---------|
| Builtins | 50% | 90% | 100% |
| Data types | 20% | 70% | 95% |
| System/OS | 30% | 60% | 90% |
| Networking | 10% | 50% | 85% |
| Text/Encoding | 10% | 40% | 80% |
| Math/Science | 40% | 60% | 90% |
| Testing | 5% | 50% | 95% |
| Development | 0% | 30% | 85% |

---

## Appendix B: Migration Guide Template

### Python to Viper: Quick Migration

```python
# Python code
from typing import List, Optional
from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int
    email: Optional[str] = None

def greet(people: List[Person]) -> List[str]:
    return [f"Hello, {p.name}!" for p in people if p.age >= 18]
```

```viper
# Viper equivalent (Phase 2+)
from typing import List, Optional
from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int
    email: Optional[str] = None

def greet(people: List[Person]) -> List[str]:
    return [f"Hello, {p.name}!" for p in people if p.age >= 18]
```

**Changes required:** None! Phase 2+ Viper accepts this Python code as-is.

---

## Appendix C: Testing Best Practices

### Writing Python-Compatible Tests

```viper
# tests/test_example.vp
import unittest
from mymodule import MyClass

class TestMyClass(unittest.TestCase):
    def setUp(self):
        self.obj = MyClass()
    
    def tearDown(self):
        self.obj.cleanup()
    
    def test_initial_state(self):
        self.assertEqual(self.obj.value, 0)
        self.assertTrue(self.obj.is_ready)
    
    def test_method_with_side_effect(self):
        result = self.obj.do_something()
        self.assertIsNotNone(result)
        self.assertIn(result, self.obj.history)
    
    def test_exception_handling(self):
        with self.assertRaises(ValueError):
            self.obj.do_invalid_operation()
    
    @unittest.skip("Not implemented yet")
    def test_future_feature(self):
        pass

if __name__ == '__main__':
    unittest.main()
```

---

## License

This roadmap is part of the Viper Language project and is distributed under the MIT License.

---

**Document Version:** 1.0  
**Last Updated:** 2026-03-01  
**Author:** Viper Development Team
