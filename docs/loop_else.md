# Loop `else` Clauses in Viper

## Overview

Viper supports Python-style `else` clauses on `for` and `while` loops. The `else` block executes when:

- **`while` loop**: The condition becomes `False` (loop terminates normally)
- **`for` loop**: The iterable is exhausted (loop completes all iterations)

The `else` block is **skipped** if the loop exits via a `break` statement.

## Syntax

### While...Else

```viper
while condition:
    # loop body
    # ...
else:
    # executes when condition becomes False
    # NOT executed if break is used
```

### For...Else

```viper
for item in iterable:
    # loop body
    # ...
else:
    # executes when iterable is exhausted
    # NOT executed if break is used
```

## Examples

### Example 1: While...Else (Normal Completion)

```viper
def main():
    i = 0
    while i < 3:
        print(i)
        i = i + 1
    else:
        print("Loop completed normally")
    
    # Output:
    # 0
    # 1
    # 2
    # Loop completed normally
```

### Example 2: While...Else (With Break)

```viper
def main():
    i = 0
    while i < 5:
        if i == 2:
            break
        print(i)
        i = i + 1
    else:
        print("This won't print - break was used")
    print("After loop")
    
    # Output:
    # 0
    # 1
    # After loop
```

### Example 3: For...Else (Normal Completion)

```viper
def main():
    for i in range(3):
        print(i)
    else:
        print("All iterations completed")
    
    # Output:
    # 0
    # 1
    # 2
    # All iterations completed
```

### Example 4: For...Else (With Break)

```viper
def main():
    for i in range(5):
        if i == 2:
            break
        print(i)
    else:
        print("This won't print - break was used")
    print("After loop")
    
    # Output:
    # 0
    # 1
    # After loop
```

### Example 5: While...Else (Condition Initially False)

```viper
def main():
    i = 10
    while i < 3:
        print(i)
        i = i + 1
    else:
        print("Else executes - body was never run")
    
    # Output:
    # Else executes - body was never run
```

## Common Use Cases

### 1. Search with Fallback

The most common pattern is searching for an item and handling the "not found" case:

```viper
def find_item(items, target):
    for item in items:
        if item == target:
            print("Found:", item)
            break
    else:
        print("Item not found")
```

### 2. Retry Logic

```viper
def retry_operation():
    attempts = 0
    max_attempts = 3
    while attempts < max_attempts:
        if try_operation():
            print("Success!")
            break
        attempts = attempts + 1
    else:
        print("All attempts failed")
```

### 3. Validation

```viper
def validate_all(items):
    for item in items:
        if not is_valid(item):
            print("Invalid item found")
            break
    else:
        print("All items are valid")
```

## Implementation Details

### Control Flow

The `else` clause implementation uses the following control flow:

```
while_loop:
    cond_block:
        if condition:
            branch to body_block
        else:
            branch to else_block (or exit_block if no else)
    
    body_block:
        execute body statements
        if break:
            branch to exit_block (skips else!)
        if continue:
            branch to cond_block
        (fall through to cond_block)
    
    else_block:
        execute else statements
        branch to exit_block
    
    exit_block:
        (continue execution)
```

### Key Points

1. **`break` skips `else`**: The `break` statement jumps directly to the exit block, bypassing the `else` block entirely.

2. **`continue` doesn't affect `else`**: The `continue` statement jumps back to the condition check, so the `else` block can still execute if the condition later becomes false.

3. **`else` executes even if body never runs**: For `while` loops, if the condition is initially false, the `else` block still executes.

4. **No `else` = no overhead**: If no `else` clause is present, the generated code is identical to before - there's no performance penalty.

## Testing

Test files are located in:
- `tests/viper_programs/test_loop_else.vp` - Comprehensive test suite
- `tests/test_loop_else.vp` - Additional test cases

Run tests with:
```bash
viper run tests/viper_programs/test_loop_else.vp
```

## Compatibility

This feature is compatible with Python's loop `else` semantics, making it easy for Python developers to transition to Viper.

## Related Features

- `break` - Exit loop early (skips `else`)
- `continue` - Skip to next iteration
- `try...else` - Execute `else` if no exception is raised
- `if...elif...else` - Conditional branching
