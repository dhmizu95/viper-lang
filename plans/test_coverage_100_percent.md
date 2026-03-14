# Plan to Achieve 100% Integration Test Coverage

**Current Status:** 66% (43/65 features covered)  
**Goal:** 100% coverage of all language features  
**Estimated Effort:** 22 new test files across 4 phases

---

## Phase 1: Critical Data Structures (Week 1)

### 1.1 List Operations (`test_lists.rs`)
**Priority:** Critical  
**Features Covered:** Index access, Slice access, List iteration, List methods

```python
# Test cases to include:
- lst = [1, 2, 3, 4, 5]
- print(lst[0])           # Index access
- print(lst[1:4])         # Slice access
- print(lst[::-1])        # Slice with step
- for x in lst:           # List iteration
    print(x)
- lst.append(6)           # List methods
- print(len(lst))
```

**Test File:** `tests/integration/lists.rs`  
**Estimated Tests:** 8-10 test functions

---

### 1.2 Dictionary Operations (`test_dicts.rs`)
**Priority:** Critical  
**Features Covered:** Dict literals, Key access, Dict methods

```python
# Test cases to include:
- d = {"a": 1, "b": 2}
- print(d["a"])           # Key access
- d["c"] = 3              # Key assignment
- print("a" in d)         # Membership test
- print(len(d))           # Dict length
- for k in d:             # Dict iteration
    print(k, d[k])
```

**Test File:** `tests/integration/dicts.rs`  
**Estimated Tests:** 6-8 test functions

---

### 1.3 List Comprehensions (`test_comprehensions.rs`)
**Priority:** Critical  
**Features Covered:** List comprehensions, Nested comprehensions, Conditional comprehensions

```python
# Test cases to include:
- squares = [x*x for x in range(10)]
- evens = [x for x in range(20) if x % 2 == 0]
- matrix = [[i*j for j in range(3)] for i in range(3)]
- flattened = [x for row in matrix for x in row]
```

**Test File:** `tests/integration/comprehensions.rs`  
**Estimated Tests:** 5-6 test functions

---

### 1.4 Tuple Operations (`test_tuples.rs`)
**Priority:** High  
**Features Covered:** Tuple literals, Tuple unpacking, Tuple indexing

```python
# Test cases to include:
- t = (1, 2, 3)
- print(t[0])             # Tuple indexing
- a, b, c = t             # Tuple unpacking
- nested = (1, (2, 3), 4)
```

**Test File:** `tests/integration/tuples.rs`  
**Estimated Tests:** 4-5 test functions

---

### 1.5 Array Literals (`test_arrays.rs`)
**Priority:** High  
**Features Covered:** Typed arrays, Array repetition, Array indexing

```python
# Test cases to include:
- arr: array[i64, 5] = [1, 2, 3, 4, 5]
- arr2 = [0] * 10         # Array repetition
- print(arr[2])           # Array indexing
```

**Test File:** `tests/integration/arrays.rs`  
**Estimated Tests:** 4-5 test functions

---

**Phase 1 Deliverables:** 5 test files, 27-34 test functions  
**Expected Coverage After Phase 1:** 78% (51/65)

---

## Phase 2: Exception Handling & Flow Control (Week 2)

### 2.1 Exception Handling (`test_exceptions.rs`)
**Priority:** Critical  
**Features Covered:** Try/except, Try/finally, Try/except/else/finally, Raise

```python
# Test cases to include:
- try:
      raise ValueError("test error")
  except ValueError as e:
      print("caught:", e)
- try:
      print("no error")
  except:
      print("error")
  else:
      print("else branch")
  finally:
      print("cleanup")
- try:
      x = 1 / 0
  except ZeroDivisionError:
      print("div by zero")
```

**Test File:** `tests/integration/exceptions.rs`  
**Estimated Tests:** 8-10 test functions

---

### 2.2 Assert & Delete (`test_assert_delete.rs`)
**Priority:** Medium  
**Features Covered:** Assert statements, Delete statement

```python
# Test cases to include:
- x = 5
  assert x > 0
  assert x < 10, "x too large"
- lst = [1, 2, 3]
  del lst[1]
  print(lst)
- d = {"a": 1, "b": 2}
  del d["a"]
  print(d)
```

**Test File:** `tests/integration/assert_delete.rs`  
**Estimated Tests:** 4-5 test functions

---

### 2.3 Walrus Operator (`test_walrus.rs`)
**Priority:** Medium  
**Features Covered:** Assignment expressions

```python
# Test cases to include:
- if (n := len(data)) > 10:
      print(f"Too many: {n}")
- while (line := read_line()) != "":
      print(line)
- result = [(y := f(x)), y**2 for x in range(10)]
```

**Test File:** `tests/integration/walrus.rs`  
**Estimated Tests:** 3-4 test functions

---

### 2.4 For-Else and While-Else (`test_loop_else.rs`)
**Priority:** Medium  
**Features Covered:** Loop else clauses with break

```python
# Test cases to include:
- for i in range(5):
      if i == 3:
          break
  else:
      print("no break")  # Should not print
- i = 0
  while i < 5:
      i += 1
  else:
      print("completed")  # Should print
```

**Test File:** `tests/integration/loop_else.rs`  
**Estimated Tests:** 4-5 test functions

---

**Phase 2 Deliverables:** 4 test files, 19-24 test functions  
**Expected Coverage After Phase 2:** 88% (57/65)

---

## Phase 3: Object-Oriented Features (Week 3)

### 3.1 Class Definitions (`test_classes.rs`)
**Priority:** Critical  
**Features Covered:** Class definition, Methods, Instance attributes, Inheritance

```python
# Test cases to include:
- class Counter:
      def __init__(self):
          self.count = 0
      def increment(self):
          self.count += 1
      def get_count(self):
          return self.count

- c = Counter()
  c.increment()
  print(c.get_count())

- class BoundedCounter(Counter):
      def __init__(self, max):
          super().__init__()
          self.max = max
```

**Test File:** `tests/integration/classes.rs`  
**Estimated Tests:** 8-10 test functions

---

### 3.2 Struct Definitions (`test_structs.rs`)
**Priority:** Critical  
**Features Covered:** Struct definition, Field access, Struct methods

```python
# Test cases to include:
- struct Point:
      x: i64
      y: i64

- p = Point(3, 4)
  print(p.x, p.y)
  p.x = 5
  print(p.x)
```

**Test File:** `tests/integration/structs.rs`  
**Estimated Tests:** 5-6 test functions

---

### 3.3 Attribute Access (`test_attributes.rs`)
**Priority:** High  
**Features Covered:** Object attribute access, Method calls, Chained attributes

```python
# Test cases to include:
- class Person:
      def __init__(self, name):
          self.name = name
      def greet(self):
          return "Hello, " + self.name

- p = Person("Alice")
  print(p.name)
  print(p.greet())
```

**Test File:** `tests/integration/attributes.rs`  
**Estimated Tests:** 4-5 test functions

---

### 3.4 Decorators Advanced (`test_decorators_advanced.rs`)
**Priority:** Medium  
**Features Covered:** Custom decorators, Decorator with arguments, Class decorators

```python
# Test cases to include:
- def timer(func):
      def wrapper(*args):
          start = time()
          result = func(*args)
          print(time() - start)
          return result
      return wrapper

- @timer
  def slow_function():
      sleep(1)
```

**Test File:** `tests/integration/decorators_advanced.rs`  
**Estimated Tests:** 4-5 test functions

---

**Phase 3 Deliverables:** 4 test files, 21-26 test functions  
**Expected Coverage After Phase 3:** 95% (62/65)

---

## Phase 4: Advanced Features (Week 4)

### 4.1 Import System (`test_imports.rs`)
**Priority:** High  
**Features Covered:** Import statement, From import, Module aliases

```python
# Test cases to include:
- import math
  print(math.sqrt(16))

- from collections import defaultdict
  d = defaultdict(int)

- import numpy as np
```

**Test File:** `tests/integration/imports.rs`  
**Estimated Tests:** 5-6 test functions  
**Note:** May require creating test modules in `tests/integration/modules/`

---

### 4.2 Match Statement (`test_match.rs`)
**Priority:** High  
**Features Covered:** Pattern matching, Multiple patterns, Guards, Wildcard

```python
# Test cases to include:
- def describe(x):
      match x:
          case 0:
              return "zero"
          case 1 | 2:
              return "small"
          case n if n < 10:
              return "medium"
          case _:
              return "large"

- match point:
      case (0, 0):
          print("origin")
      case (x, 0):
          print(f"on x-axis: {x}")
      case (0, y):
          print(f"on y-axis: {y}")
      case (x, y):
          print(f"point: ({x}, {y})")
```

**Test File:** `tests/integration/match.rs`  
**Estimated Tests:** 6-8 test functions

---

### 4.3 Concurrency - Channels (`test_channels.rs`)
**Priority:** High  
**Features Covered:** Channel creation, Send, Receive, Select

```python
# Test cases to include:
- ch = chan(10)
  send(ch, 42)
  result = recv(ch)
  print(result)

- def sender(ch):
      for i in range(5):
          send(ch, i)

- def receiver(ch):
      for i in range(5):
          print(recv(ch))

- ch = chan()
  task: sender(ch)
  receiver(ch)
```

**Test File:** `tests/integration/channels.rs`  
**Estimated Tests:** 6-8 test functions

---

### 4.4 Concurrency - WaitGroup (`test_waitgroup.rs`)
**Priority:** Medium  
**Features Covered:** WaitGroup creation, Add, Done, Wait

```python
# Test cases to include:
- wg = WaitGroup()
  for i in range(5):
      wg.add(1)
      task: worker(i, wg)
  wg.wait()

- def worker(id, wg):
      print("worker", id)
      wg.done()
```

**Test File:** `tests/integration/waitgroup.rs`  
**Estimated Tests:** 3-4 test functions

---

### 4.5 With Statement (`test_with.rs`)
**Priority:** Medium  
**Features Covered:** Context managers, With statement, Multiple context managers

```python
# Test cases to include:
- with open("test.txt") as f:
      content = f.read()
      print(content)

- with lock:
      critical_section()

- with open("in.txt") as fin, open("out.txt", "w") as fout:
      fout.write(fin.read())
```

**Test File:** `tests/integration/with.rs`  
**Estimated Tests:** 4-5 test functions

---

### 4.6 Generators (`test_generators.rs`)
**Priority:** Low  
**Features Covered:** Yield statement, Generator functions, Generator iteration

```python
# Test cases to include:
- def countdown(n):
      while n > 0:
          yield n
          n -= 1

- for i in countdown(5):
      print(i)

- def fibonacci():
      a, b = 0, 1
      while True:
          yield a
          a, b = b, a + b
```

**Test File:** `tests/integration/generators.rs`  
**Estimated Tests:** 4-5 test functions

---

### 4.7 Raise Statement (`test_raise.rs`)
**Priority:** Medium  
**Features Covered:** Raise exceptions, Raise with cause, Re-raise

```python
# Test cases to include:
- raise ValueError("error message")

- try:
      x = 1 / 0
  except ZeroDivisionError as e:
      raise ValueError("invalid operation") from e

- def validate(x):
      if x < 0:
          raise ValueError("negative")
      return x
```

**Test File:** `tests/integration/raise.rs`  
**Estimated Tests:** 4-5 test functions

---

**Phase 4 Deliverables:** 7 test files, 32-41 test functions  
**Expected Coverage After Phase 4:** 100% (65/65)

---

## Summary

### Test Files to Create

| Phase | Files | Tests | Features Added | Coverage |
|-------|-------|-------|----------------|----------|
| Phase 1 | 5 | 27-34 | Lists, Dicts, Comprehensions, Tuples, Arrays | 78% |
| Phase 2 | 4 | 19-24 | Exceptions, Assert, Delete, Walrus, Loop-else | 88% |
| Phase 3 | 4 | 21-26 | Classes, Structs, Attributes, Advanced decorators | 95% |
| Phase 4 | 7 | 32-41 | Imports, Match, Channels, WaitGroup, With, Generators, Raise | 100% |
| **Total** | **20** | **99-125** | **All features** | **100%** |

### Implementation Order

```
Week 1: Phase 1 (Data Structures)
  - test_lists.rs
  - test_dicts.rs
  - test_comprehensions.rs
  - test_tuples.rs
  - test_arrays.rs

Week 2: Phase 2 (Exception Handling & Flow)
  - test_exceptions.rs
  - test_assert_delete.rs
  - test_walrus.rs
  - test_loop_else.rs

Week 3: Phase 3 (Object-Oriented)
  - test_classes.rs
  - test_structs.rs
  - test_attributes.rs
  - test_decorators_advanced.rs

Week 4: Phase 4 (Advanced Features)
  - test_imports.rs
  - test_match.rs
  - test_channels.rs
  - test_waitgroup.rs
  - test_with.rs
  - test_generators.rs
  - test_raise.rs
```

### Testing Guidelines

1. **Test File Structure:**
   ```rust
   //! Feature description

   use crate::utils::run_viper_code;

   #[test]
   fn test_feature_basic() {
       let code = r#"
   # Test code here
   "#;
       assert!(run_viper_code(code).is_ok());
   }
   ```

2. **Each Test Should:**
   - Test one specific feature
   - Include edge cases
   - Verify output with `print()` statements
   - Include error cases where applicable

3. **Code Style:**
   - Use descriptive test names: `test_feature_scenario()`
   - Include comments explaining what's being tested
   - Keep test code concise but comprehensive

### Success Criteria

- [ ] All 20 test files created
- [ ] All tests pass with `cargo test`
- [ ] No pre-existing tests broken
- [ ] Coverage report shows 100%
- [ ] Documentation updated with examples

---

**Created:** 2026-03-14  
**Status:** Pending Implementation  
**Owner:** Development Team
