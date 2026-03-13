# Viper Async/Await Quick Reference

## Syntax Comparison

### task/sync (Existing - Unchanged)

```python
# Fire-and-forget parallel work
def worker(i):
    result = compute(i)
    store_result(result)

def main():
    for i in range(1000):
        task worker(i)   # Spawn concurrent task
    sync:               # Wait for all to complete
        pass
```

**Use when:**
- No return value needed
- Fire-and-forget parallel work
- Simple concurrency

---

### async/await (New - Fiber-Based)

```python
# With results and suspension
async def worker(i):
    await sleep(10)      # Suspend, let others run
    return compute(i)

async def main():
    futures = []
    for i in range(1000):
        f = worker(i)     # Returns Future, doesn't run yet
        futures.append(f)
    
    results = []
    for f in futures:
        r = await f       # Suspend until ready
        results.append(r)
    
    print(sum(results))

main()
```

**Use when:**
- Need return values
- I/O-bound operations
- Pipelined processing

---

## Common Patterns

### Pattern 1: Parallel Map

```python
# task/sync - no results
results = []
def mapper(i):
    global results
    results[i] = compute(i)

for i in range(n):
    task mapper(i)
sync:
    pass

# async/await - with results
async def mapper(i):
    return compute(i)

futures = [mapper(i) for i in range(n)]
results = [await f for f in futures]
```

### Pattern 2: Pipeline

```python
# async/await only - staged processing
async def fetch(url):
    return await http_get(url)

async def transform(data):
    return process(data)

async def store(result):
    await db_insert(result)

async def pipeline(url):
    data = await fetch(url)
    processed = await transform(data)
    await store(processed)
    return processed
```

### Pattern 3: Concurrent I/O

```python
# async/await - all run concurrently
async def fetch_all(urls):
    futures = [http_get(u) for u in urls]
    return [await f for f in futures]

# vs sequential
async def fetch_sequential(urls):
    results = []
    for url in urls:
        r = await http_get(url)  # One at a time
        results.append(r)
    return results
```

### Pattern 4: Async For

```python
# Stream processing
async def process_stream():
    total = 0
    async for item in async_range(1000):
        result = await process(item)
        total += result
    return total
```

---

## API Reference

### Keywords

| Keyword | Usage | Description |
|---------|-------|-------------|
| `async def` | `async def foo():` | Define async function |
| `await` | `result = await future` | Wait for future, yields fiber |
| `async for` | `async for x in iter:` | Iterate async iterator |
| `task` | `task worker()` | Spawn fire-and-forget task |
| `sync` | `sync: pass` | Wait for all tasks |

### Built-in Async Functions

| Function | Description |
|----------|-------------|
| `async_range(n)` | Async iterator 0..n-1 |
| `sleep(ms)` | Suspend for milliseconds |
| `gather(f1, f2, ...)` | Wait for multiple futures (future) |

### Future Methods

```python
# Future[T] - returned by async function call
f = async_func()

# Await to get result
result = await f

# Check if ready (future)
if f.is_ready():
    result = f.result()
```

---

## Performance

| Model | Tasks | Memory | Use Case |
|-------|-------|--------|----------|
| `task`/`sync` | 10M+ | ~100 bytes/task | Parallel CPU work |
| `async`/`await` | 10M+ | ~150 bytes/task | I/O with results |
| OS threads | ~10K | ~1 MB/thread | Blocking I/O |
| Python asyncio | ~100K | ~50 KB/task | Event loop I/O |

---

## Migration Guide

### From Python asyncio

```python
# Python
import asyncio

async def fetch(url):
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as resp:
            return await resp.text()

async def main():
    urls = [...]
    tasks = [asyncio.create_task(fetch(u)) for u in urls]
    results = await asyncio.gather(*tasks)

asyncio.run(main())

# Viper
async def fetch(url):
    # HTTP client would be implemented similarly
    return await http_get(url)

async def main():
    urls = [...]
    futures = [fetch(u) for u in urls]
    results = [await f for f in futures]

main()
```

### From Go goroutines

```go
// Go
func worker(id int, wg *sync.WaitGroup) {
    defer wg.Done()
    // work
}

func main() {
    var wg sync.WaitGroup
    for i := 0; i < 1000; i++ {
        wg.Add(1)
        go worker(i, &wg)
    }
    wg.Wait()
}
```

```python
# Viper task/sync (similar)
def worker(id):
    # work

def main():
    for i in range(1000):
        task worker(i)
    sync:
        pass

# Viper async/await (with results)
async def worker(id):
    # work
    return result

async def main():
    futures = [worker(i) for i in range(1000)]
    results = [await f for f in futures]
```

---

## Error Handling

```python
async def risky():
    if condition:
        raise Error("something went wrong")
    return 42

async def main():
    try:
        result = await risky()
    except Error as e:
        print("Error:", e)
        result = 0
    return result
```

---

## Best Practices

1. **Use `task`/`sync` for simple parallel work** - No results needed
2. **Use `async`/`await` for I/O** - Network, disk, databases
3. **Create all futures first, then await** - Maximizes concurrency
4. **Avoid `await` in loops for concurrent execution**:
   ```python
   # Bad - sequential
   for url in urls:
       result = await fetch(url)
   
   # Good - concurrent
   futures = [fetch(url) for url in urls]
   results = [await f for f in futures]
   ```
5. **Use `async for` for streams** - Async generators, pagination
