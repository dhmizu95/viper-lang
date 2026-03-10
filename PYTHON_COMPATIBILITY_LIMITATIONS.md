# Viper Language - Python Compatibility Limitations

This document details the dynamic features that are **not fully supported** in Viper compared to CPython.

---

## Table of Contents

1. [Dynamic Features Not Supported](#dynamic-features-not-supported)
2. [Workarounds](#workarounds)
3. [What Does Work](#what-does-work)

---

## Dynamic Features Not Supported

### 1. `eval()` and `exec()` - Limited Support

**Python:**
```python
# Dynamic code execution
code = "x = 1 + 1"
exec(code)
print(x)  # 2

result = eval("2 + 2")
print(result)  # 4
```

**Viper Status:** ⚠️ **Limited**

Viper is a compiled language with static type checking. Runtime code execution is limited because:
- Types must be known at compile time
- No runtime AST manipulation
- Security concerns with arbitrary code execution

**Workaround:** Use functions instead of dynamic code
```viper
# Instead of eval("2 + 2")
def calculate():
    return 2 + 2

# Instead of exec("x = 1")
def setup():
    x = 1
    return x
```

---

### 2. Runtime Attribute Modification (Monkey Patching)

**Python:**
```python
class MyClass:
    def original_method(self):
        return "original"

obj = MyClass()

# Add method at runtime
def new_method(self):
    return "patched"

MyClass.new_method = new_method
print(obj.new_method())  # "patched"

# Replace existing method
MyClass.original_method = new_method
print(obj.original_method())  # "patched"
```

**Viper Status:** ❌ **Not Supported**

Viper classes are fixed at compile time:
- Cannot add methods after class definition
- Cannot replace methods at runtime
- Method resolution is determined at compile time

**Workaround:** Use composition or strategy pattern
```viper
class MyClass:
    def __init__(self, strategy):
        self.strategy = strategy
    
    def execute(self):
        return self.strategy.run()

class Strategy1:
    def run(self):
        return "strategy1"

class Strategy2:
    def run(self):
        return "strategy2"

# Switch strategies instead of monkey patching
obj = MyClass(Strategy1())
obj.execute()  # "strategy1"
obj.strategy = Strategy2()
obj.execute()  # "strategy2"
```

---

### 3. Dynamic Attribute Access with `__getattr__`

**Python:**
```python
class DynamicClass:
    def __getattr__(self, name):
        # Handle any attribute dynamically
        return f"dynamic_{name}"

obj = DynamicClass()
print(obj.anything)      # "dynamic_anything"
print(obj.something_else) # "dynamic_something_else"
```

**Viper Status:** ❌ **Not Supported**

Viper requires explicit attribute definitions:
- All attributes must be declared
- No dynamic attribute creation
- Type checker needs to know all attributes

**Workaround:** Use dictionaries for dynamic data
```viper
class DynamicData:
    def __init__(self):
        self._data = {}
    
    def get(self, key):
        return self._data.get(key)
    
    def set(self, key, value):
        self._data[key] = value

obj = DynamicData()
obj.set("anything", "value1")
obj.set("something_else", "value2")
print(obj.get("anything"))
```

---

### 4. Metaclasses

**Python:**
```python
class Meta(type):
    def __new__(cls, name, bases, dct):
        # Modify class creation
        dct['custom_attr'] = True
        return super().__new__(cls, name, bases, dct)

class MyClass(metaclass=Meta):
    pass

print(MyClass.custom_attr)  # True
```

**Viper Status:** ❌ **Not Supported**

Viper doesn't support metaclasses because:
- Class structure is fixed at compile time
- No runtime class modification
- Type system requires known class structure

**Workaround:** Use class decorators (if implemented) or factory functions
```viper
def create_class_with_attr(name, attr_value):
    # Factory function instead of metaclass
    class GeneratedClass:
        custom_attr = attr_value
    
    return GeneratedClass

MyClass = create_class_with_attr("MyClass", True)
```

---

### 5. `__slots__` Optimization

**Python:**
```python
class OptimizedClass:
    __slots__ = ['x', 'y']  # Prevent __dict__ creation
    
    def __init__(self, x, y):
        self.x = x
        self.y = y
```

**Viper Status:** ⚠️ **Not Needed**

Viper uses ARC (Atomic Reference Counting) for memory management:
- No `__dict__` overhead by default
- Fields are statically allocated
- Memory optimization is automatic

---

### 6. Descriptors (`__get__`, `__set__`, `__delete__`)

**Python:**
```python
class Descriptor:
    def __get__(self, obj, objtype=None):
        return "descriptor_value"
    
    def __set__(self, obj, value):
        pass

class MyClass:
    attr = Descriptor()

obj = MyClass()
print(obj.attr)  # "descriptor_value"
```

**Viper Status:** ❌ **Not Supported**

Viper doesn't support descriptor protocol:
- No `__get__`, `__set__`, `__delete__` hooks
- Attribute access is direct

**Workaround:** Use properties
```viper
class MyClass:
    def __init__(self):
        self._attr = "value"
    
    def get_attr(self) -> str:
        return self._attr
    
    def set_attr(self, value):
        self._attr = value
```

---

### 7. Import Hooks

**Python:**
```python
import sys

class CustomImporter:
    def find_module(self, name, path=None):
        return self
    
    def load_module(self, name):
        # Custom module loading logic
        pass

sys.meta_path.insert(0, CustomImporter())
```

**Viper Status:** ❌ **Not Supported**

Viper's module system is static:
- Modules are resolved at compile time
- No runtime import customization
- Import paths are fixed

---

### 8. `__import__` Override

**Python:**
```python
import builtins

original_import = builtins.__import__

def custom_import(name, *args, **kwargs):
    print("Importing:", name)
    return original_import(name, *args, **kwargs)

builtins.__import__ = custom_import
```

**Viper Status:** ❌ **Not Supported**

Viper's import system cannot be overridden:
- `import` statements are compile-time directives
- No runtime import hook

---

### 9. Frame Inspection (`sys._getframe()`)

**Python:**
```python
import sys

def get_caller_name():
    frame = sys._getframe(1)
    return frame.f_code.co_name

def caller():
    return get_caller_name()

print(caller())  # "caller"
```

**Viper Status:** ❌ **Not Supported**

Viper doesn't expose frame objects:
- No `sys._getframe()`
- No `f_locals`, `f_globals` access
- Call stack is not introspectable

**Workaround:** Pass context explicitly
```viper
def log(message, caller_name):
    print(caller_name, ":", message)

def caller():
    log("Hello", "caller")
```

---

### 10. Garbage Collection Hooks (`__del__`, `gc` module)

**Python:**
```python
class MyClass:
    def __del__(self):
        print("Object destroyed")

import gc
gc.collect()  # Force garbage collection
```

**Viper Status:** ⚠️ **Different Model**

Viper uses ARC (Atomic Reference Counting):
- No `__del__` method
- No `gc` module
- Objects are freed immediately when ref count reaches 0
- No cyclic GC (cycles are detected but not collected automatically)

---

### 11. Weak References (`weakref` module)

**Python:**
```python
import weakref

class MyClass:
    pass

obj = MyClass()
weak = weakref.ref(obj)
print(weak())  # <MyClass object>
del obj
print(weak())  # None
```

**Viper Status:** ❌ **Not Implemented**

Viper doesn't have weak references:
- All references are strong (ARC)
- No `weakref` module
- Circular references may leak memory

---

### 12. Coroutines with `async`/`await` - Limited

**Python:**
```python
async def fetch_data():
    await asyncio.sleep(1)
    return "data"

async def main():
    result = await fetch_data()
    print(result)

asyncio.run(main())
```

**Viper Status:** ⚠️ **Partial**

Viper has `async`/`await` syntax but:
- Event loop implementation may be incomplete
- Some async stdlib modules not available
- `asyncio` module may have limited functionality

---

## What Does Work ✅

### Fully Supported Features

| Feature | Status | Notes |
|---------|--------|-------|
| Basic syntax | ✅ | Indentation, blocks, etc. |
| Functions | ✅ | All features |
| Classes | ✅ | Inheritance, methods |
| @dataclass | ✅ | Full support |
| Type hints | ✅ | Full support |
| Generics | ✅ | TypeVar, Generic |
| Union types | ✅ | `int | str` |
| Context managers | ✅ | `with` statement |
| Exceptions | ✅ | try/except/finally |
| Iterators | ✅ | `__iter__`, `__next__` |
| List comprehensions | ✅ | Full support |
| Decorators | ✅ | @staticmethod, @classmethod, @property |
| Multiple inheritance | ✅ | C3 MRO |
| Operator overloading | ✅ | `__add__`, `__eq__`, etc. |

### Standard Library Support

| Module | Status | Notes |
|--------|--------|-------|
| `math` | ✅ | Full support |
| `json` | ✅ | Full support |
| `re` | ✅ | Full support |
| `random` | ✅ | Full support |
| `collections` | ✅ | namedtuple, OrderedDict, etc. |
| `functools` | ✅ | partial, reduce, lru_cache |
| `itertools` | ✅ | All functions |
| `datetime` | ✅ | date, time, datetime, timedelta |
| `csv` | ✅ | reader, writer, DictReader |
| `io` | ✅ | StringIO, BytesIO |
| `pathlib` | ✅ | Path class |
| `string` | ✅ | Template, constants |
| `contextlib` | ✅ | contextmanager, suppress |
| `typing` | ✅ | TypeVar, Generic, etc. |
| `unittest` | ✅ | Full framework |
| `unittest.mock` | ✅ | Mock, MagicMock, patch |
| `coverage` | ✅ | Coverage analysis |
| `pdb` | ✅ | Debugger |

---

## Migration Tips

### When Porting Python Code to Viper

1. **Replace `eval()`/`exec()`** with functions
2. **Replace monkey patching** with composition
3. **Replace dynamic attributes** with dictionaries
4. **Replace metaclasses** with factory functions
5. **Replace descriptors** with properties
6. **Replace `__del__`** with explicit cleanup methods
7. **Replace weak references** with strong references
8. **Replace frame inspection** with explicit context passing

### Example Migration

**Python:**
```python
class Config:
    def __getattr__(self, name):
        return os.environ.get(name)

config = Config()
print(config.DATABASE_URL)
```

**Viper:**
```viper
class Config:
    def __init__(self):
        self._env = {}
    
    def get(self, name: str) -> str:
        return self._env.get(name, "")
    
    def set(self, name: str, value: str):
        self._env[name] = value

config = Config()
print(config.get("DATABASE_URL"))
```

---

## Summary

| Category | Supported | Limited | Not Supported |
|----------|-----------|---------|---------------|
| Core syntax | ✅ 100% | | |
| Type system | ✅ 98% | | |
| OOP | ✅ 95% | | Descriptors, metaclasses |
| Dynamic features | | ⚠️ eval/exec | ❌ Monkey patching, `__getattr__` |
| Introspection | | ⚠️ type(), isinstance() | ❌ Frames, `__import__` |
| Memory management | ✅ ARC | | ❌ `__del__`, weakref |
| Stdlib | ✅ 65%+ | ⚠️ asyncio | ❌ C extensions |

**Viper is best suited for:**
- Static, type-safe code
- Performance-critical applications
- Code that doesn't rely on dynamic features

**Consider CPython for:**
- Heavy use of `eval()`/`exec()`
- Dynamic attribute manipulation
- C extension integration
- Metaprogramming with metaclasses
