//! Integration tests for async/await with fiber-based implementation

use crate::utils::run_viper_code;

#[test]
fn test_async_def_returns_future() {
    let code = r#"
async def simple():
    return 42

async def main():
    f = simple()
    print("future:", f)
    result = await f
    print("result:", result)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("result: 42"));
}

#[test]
fn test_await_multiple_futures() {
    let code = r#"
async def worker(n):
    return n * 2

async def main():
    f1 = worker(1)
    f2 = worker(2)
    f3 = worker(3)
    
    r1 = await f1
    r2 = await f2
    r3 = await f3
    
    print("results:", r1, r2, r3)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("results: 2 4 6"));
}

#[test]
fn test_async_with_args() {
    let code = r#"
async def add(a, b):
    return a + b

async def main():
    result = await add(10, 20)
    print("sum:", result)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("sum: 30"));
}

#[test]
fn test_async_concurrent_execution() {
    let code = r#"
async def worker(n):
    # Simulate work
    result = 0
    for i in range(1000):
        result = result + i
    return n * 100 + result

async def main():
    futures = []
    for i in range(10):
        futures.append(worker(i))
    
    total = 0
    for f in futures:
        r = await f
        total = total + r
    
    print("total:", total)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("total:"));
}

#[test]
fn test_async_for_range() {
    let code = r#"
async def main():
    total = 0
    async for i in async_range(10):
        total = total + i
    print("total:", total)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("total: 45"));
}

#[test]
fn test_async_for_with_await() {
    let code = r#"
async def process(n):
    await sleep(1)
    return n * 2

async def main():
    results = []
    async for i in async_range(5):
        r = await process(i)
        results.append(r)
    print("results:", results)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("results: [0, 2, 4, 6, 8]"));
}

#[test]
fn test_async_sleep() {
    let code = r#"
async def sleeper(n):
    await sleep(10)
    return n

async def main():
    futures = []
    for i in range(5):
        futures.append(sleeper(i))
    
    results = []
    for f in futures:
        r = await f
        results.append(r)
    
    print("done:", results)

main()
"#;
    
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("done:"));
}
