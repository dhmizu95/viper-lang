# Viper Examples and Benchmarks

A comprehensive guide to Viper language examples and performance benchmarks.

## Table of Contents

1. [Basic Examples](#basic-examples)
2. [Algorithm Examples](#algorithm-examples)
3. [Concurrency Examples](#concurrency-examples)
4. [Benchmark Suite](#benchmark-suite)
5. [Performance Tips](#performance-tips)

---

## Basic Examples

### Hello World

The simplest Viper program:

```python
# hello.vp
print("Hello, World!")
```

**Run:**
```bash
viper run hello.vp
```

### Variables and Types

```python
# variables.vp
# Variable declarations with type inference
name = "Viper"
version = 1.0
count = 42
is_awesome = True

# Explicit type annotations
age: i64 = 10
pi: f64 = 3.14159

# Print with variables
print("Name:", name)
print("Version:", version)
print("Count:", count)
print("Age:", age)
print("Pi:", pi)
```

### Operators

```python
# operators.vp
# Arithmetic
a = 10 + 5    # 15
b = 10 - 5    # 5
c = 10 * 5    # 50
d = 10 / 5    # 2.0
e = 10 % 3    # 1
f = 2 ** 10   # 1024

# Comparison
print(10 > 5)   # True
print(10 == 10) # True
print(10 != 5)  # True

# Logical
print(True and False)  # False
print(True or False)  # True
print(not True)       # False

# String concatenation
greeting = "Hello, " + "World!"
print(greeting)
```

---

## Algorithm Examples

### Factorial

```python
# factorial.vp
def factorial(n: i64) -> i64:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def factorial_iterative(n: i64) -> i64:
    result = 1
    for i in range(2, n + 1):
        result = result * i
    return result

print("Factorial (recursive):")
for i in range(10):
    print(i, "! =", factorial(i))

print("\nFactorial (iterative):")
for i in range(10):
    print(i, "! =", factorial_iterative(i))
```

**Run:**
```bash
viper run benchmark/16_factorial/factorial.vp
```

### Fibonacci

```python
# fibonacci.vp
def fib(n: i64) -> i64:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def fib_iterative(n: i64) -> i64:
    if n <= 1:
        return n
    a, b = 0, 1
    for i in range(n - 1):
        a, b = b, a + b
    return b

print("Fibonacci (recursive):")
for i in range(10):
    print("fib(", i, ") =", fib(i))

print("\nFibonacci (iterative):")
for i in range(10):
    print("fib(", i, ") =", fib_iterative(i))
```

**Run:**
```bash
viper run benchmark/02_fibonacci/fibonacci.vp
```

### Prime Sieve

```python
# sieve.vp
def sieve(n: i64) -> i64:
    # Create sieve array using list comprehension pattern
    is_prime = [True] * (n + 1)
    is_prime[0] = False
    is_prime[1] = False
    
    # Mark multiples of each prime as composite
    for i in range(2, int(sqrt(n)) + 1):
        if is_prime[i]:
            # Mark all multiples of i starting from i*i
            for j in range(i * i, n + 1, i):
                is_prime[j] = False
    
    # Count primes
    count = 0
    for i in range(2, n + 1):
        if is_prime[i]:
            count = count + 1
    
    return count

def main():
    print("Prime Sieve Benchmark")
    n = 100000
    result = sieve(n)
    print("Primes below", n, "=", result)

main()
```

**Run:**
```bash
viper run benchmark/01_prime_sieve/sieve.vp
```

### Quicksort

```python
# quicksort.vp
def swap(arr, i, j):
    temp = arr[i]
    arr[i] = arr[j]
    arr[j] = temp

def partition(arr, low, high):
    pivot = arr[high]
    i = low - 1
    for j in range(low, high):
        if arr[j] <= pivot:
            i = i + 1
            swap(arr, i, j)
    swap(arr, i + 1, high)
    return i + 1

def quickSort(arr, low, high):
    if low < high:
        pi = partition(arr, low, high)
        quickSort(arr, low, pi - 1)
        quickSort(arr, pi + 1, high)

def main():
    SIZE = 5000
    
    # Create unsorted array
    arr = []
    for i in range(SIZE, 0, -1):
        arr.append(i)
    
    # Sort
    quickSort(arr, 0, SIZE - 1)
    
    # Print first 10
    print("Sorted (first 10):")
    for i in range(10):
        print(arr[i])

main()
```

**Run:**
```bash
viper run benchmark/04_quicksort/quicksort.vp
```

### Binary Search

```python
# binary_search.vp
def binary_search(arr: [i64], target: i64) -> i64:
    left = 0
    right = len(arr) - 1
    
    while left <= right:
        mid = (left + right) / 2
        if arr[mid] == target:
            return mid
        elif arr[mid] < target:
            left = mid + 1
        else:
            right = mid - 1
    
    return -1

def main():
    # Create sorted array
    arr = []
    for i in range(100):
        arr.append(i * 2)  # [0, 2, 4, ..., 198]
    
    # Search
    result = binary_search(arr, 50)
    print("Index of 50:", result)
    
    result = binary_search(arr, 51)
    print("Index of 51:", result)  # -1 (not found)

main()
```

---

## Concurrency Examples

### Channel Basics

```python
# channel_basic.vp
def main():
    # Create channel with buffer
    c = chan(10)
    
    # Send values
    send(c, 42)
    send(c, 100)
    send(c, 256)
    
    # Receive values
    v1 = recv(c)
    v2 = recv(c)
    v3 = recv(c)
    
    print("Received:", v1, v2, v3)

main()
```

### Task Spawning

```python
# task_spawn.vp
def worker(id: i64, output: chan[i64]):
    result = id * id
    send(output, result)

def main():
    result_chan = chan(10)
    
    # Spawn 5 workers
    for i in range(5):
        task worker(i, result_chan)
    
    # Collect results
    results = []
    for i in range(5):
        results.append(recv(result_chan))
    
    print("Results:", results)

main()
```

### WaitGroup Synchronization

```python
# waitgroup.vp
def worker(id: i64, wg: WaitGroup):
    print("Worker", id, "starting")
    # Simulate work
    result = id * 2
    print("Worker", id, "done")
    wg.done()

def main():
    wg = WaitGroup()
    wg.add(3)
    
    # Spawn workers
    task worker(0, wg)
    task worker(1, wg)
    task worker(2, wg)
    
    # Wait for completion
    wg.wait()
    print("All workers completed")

main()
```

### Select Statement

```python
# select_stmt.vp
def main():
    chan1 = chan(5)
    chan2 = chan(5)
    
    # Send initial values
    send(chan1, 1)
    send(chan2, 2)
    
    # Use select to receive from either channel
    select:
        case v = recv(chan1):
            print("Received from chan1:", v)
        case chan2 <- 100:
            print("Sent to chan2")
        case timeout(1000):
            print("Timeout")

main()
```

### Producer-Consumer

```python
# producer_consumer.vp
def producer(jobs: chan[i64], count: i64):
    for i in range(count):
        send(jobs, i)
    close(jobs)

def consumer(id: i64, jobs: chan[i64], results: chan[i64]):
    count = 0
    while True:
        select:
            case job = recv(jobs):
                count = count + 1
            case timeout(100):
                break
    send(results, count)

def main():
    jobs = chan(100)
    results = chan(5)
    
    # Start producer
    task producer(jobs, 100)
    
    # Start consumers
    for i in range(3):
        task consumer(i, jobs, results)
    
    # Collect results
    total = 0
    for i in range(3):
        total = total + recv(results)
    
    print("Total items processed:", total)

main()
```

---

## Benchmark Suite

The Viper benchmark suite compares performance against C, Rust, and Go.

### Running Benchmarks

```bash
# Navigate to benchmark directory
cd benchmark

# Build all
./build_all.sh

# Run all benchmarks
./run_all.sh

# Run individual benchmarks
cd 01_prime_sieve
./run_comparison.sh
```

### Benchmark Categories

#### 01. Prime Sieve

Measures integer arithmetic and array operations.

```bash
viper run benchmark/01_prime_sieve/sieve.vp
# Expected output: Primes below 100000 = 9592
```

#### 02. Fibonacci

Measures recursion and function call overhead.

```bash
viper run benchmark/02_fibonacci/fibonacci.vp
```

#### 03. Matrix Multiplication

Measures floating-point performance.

```bash
viper run benchmark/03_matrix_multiply/matrix_mul.vp
```

#### 04. Quicksort

Measures sorting algorithm performance.

```bash
viper run benchmark/04_quicksort/quicksort.vp
```

#### 05. Mandelbrot Set

Measures floating-point and loop performance.

```bash
viper run benchmark/05_mandelbrot/mandelbrot.vp
```

#### 06. Raytracer

Measures object-oriented code performance.

```bash
viper run benchmark/06_raytracer/raytracer.vp
```

#### 07. N-Body Simulation

Measures physics simulation performance.

```bash
viper run benchmark/07_nbody/nbody.vp
```

#### 08. Binary Trees

Measures pointer/chasing performance.

```bash
viper run benchmark/08_binary_trees/binary_trees.vp
```

#### 09. Fannkuch

Measures array permutation performance.

```bash
viper run benchmark/09_fannkuch/fannkuch.vp
```

#### 10. Spectral Norm

Measures numerical computing performance.

```bash
viper run benchmark/10_spectral_norm/spectral_norm.vp
```

### BigInt Benchmarks

For large number calculations requiring arbitrary precision:

```bash
# Fibonacci with BigInt
viper run benchmark/17_fibonacci_big/fibonacci.vp

# Factorial
viper run benchmark/16_factorial/factorial.vp
```

### Performance Comparison Results

| Benchmark | Viper (JIT) | Viper (AOT) | C | Rust | Go |
|-----------|-------------|-------------|---|---|---|
| Prime Sieve 100K | ~X ms | ~X ms | ~X ms | ~X ms | ~X ms |
| Fibonacci 20 | ~X ms | ~X ms | ~X ms | ~X ms | ~X ms |
| Matrix Multiply | ~X ms | ~X ms | ~X ms | ~X ms | ~X ms |

*See [benchmark/RESULTS.md](../benchmark/RESULTS.md) for detailed results*

---

## Performance Tips

### 1. Pre-allocate Lists

```python
# Fast: Single allocation
data = [0] * 1000000

# Slow: Many allocations
data = []
for i in range(1000000):
    data.append(i)
```

### 2. Use Arrays for Fixed Sizes

```python
# Fast: Stack-allocated
nums: [i64; 1000] = [0; 1000]

# Slower: Heap-allocated
nums = [0] * 1000
```

### 3. Choose Appropriate Optimization Level

```bash
# Development (fast compile)
viper run program.vp

# Production (optimized)
viper build program.vp -O 3 -o program
```

### 4. Use Built-in Functions

```python
# Fast: Built-in
total = sum(my_list)

# Slower: Manual loop
total = 0
for x in my_list:
    total = total + x
```

### 5. Avoid Global Variables

```python
# Faster: Pass as parameter
def process(data: [i64]) -> i64:
    return sum(data)

# Slower: Global access
global_data = [1, 2, 3]
def process():
    return sum(global_data)
```

---

## File Structure

```
benchmark/
├── 01_prime_sieve/       # Prime Sieve of Eratosthenes
├── 02_fibonacci/         # Fibonacci calculations
├── 03_matrix_multiply/  # Matrix multiplication
├── 04_quicksort/        # Quicksort algorithm
├── 05_mandelbrot/       # Mandelbrot set rendering
├── 06_raytracer/        # Simple raytracer
├── 07_nbody/            # N-body simulation
├── 08_binary_trees/     # Binary tree operations
├── 09_fannkuch/         # Fannkuch benchmark
├── 10_spectral_norm/   # Spectral norm calculation
├── 11_k_nucleotide/    # k-nucleotide
├── 12_reverse_complement/  # DNA reverse complement
├── 13_regex_dna/        # Regex DNA analysis
├── 14_champernowne/    # Champernowne's constant
├── 15_euler_sum/       # Summation of powers
├── 16_factorial/       # Factorial (BigInt)
├── 17_fibonacci_big/   # Fibonacci (BigInt)
├── 26_monte_carlo_pi/  # Monte Carlo Pi
├── 41_matrix_1000/     # 1000x1000 matrix multiply
├── 66_bfs/             # Breadth-first search
├── bigint/             # BigInt benchmarks
└── insert_1m_*.vp      # List insertion benchmarks
```

---

## See Also

- [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md) - Complete language reference
- [STDLIB_REFERENCE.md](STDLIB_REFERENCE.md) - Standard library reference
- [benchmark/README.md](../benchmark/README.md) - Benchmark suite details
- [benchmark/RESULTS.md](../benchmark/RESULTS.md) - Performance results
