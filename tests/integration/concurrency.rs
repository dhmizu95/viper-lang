//! Concurrency Integration Tests
//! Covers: task spawn, sync block, chan, send, recv, select, WaitGroup

use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let _ = fs::remove_file(&test_file);
    if !output.status.success() {
        return Err(format!("stdout: {}\nstderr: {}", stdout, stderr));
    }
    Ok(stdout)
}

// ============================================================================
// Task Spawn + Sync Block
// ============================================================================

#[test]
fn test_task_simple_spawn() {
    let code = r#"
result = 0

def worker():
    global result
    result = 42

def test():
    global result
    sync:
        task worker()
    print(result)
test()
"#;
    let output = run_viper_code(code).expect("simple task spawn should work");
    assert!(output.contains("42"), "got: {}", output);
}

#[test]
fn test_task_multiple_workers() {
    let code = r#"
counter = 0

def bump():
    global counter
    counter += 1

def test():
    global counter
    sync:
        task bump()
        task bump()
        task bump()
    print(counter)
test()
"#;
    let output = run_viper_code(code).expect("multiple tasks should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_sync_runs_all_tasks() {
    let code = r#"
log = []

def add_item(x):
    log.append(x)

def test():
    sync:
        task add_item(1)
        task add_item(2)
        task add_item(3)
    log.sort()
    print(len(log))
test()
"#;
    let output = run_viper_code(code).expect("sync block should complete all tasks");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_task_with_computation() {
    let code = r#"
total = 0

def compute(n):
    global total
    result = 0
    for i in range(n):
        result = result + i
    total += result

def test():
    global total
    sync:
        task compute(10)
    print(total)
test()
"#;
    let output = run_viper_code(code).expect("task with computation should work");
    // 0+1+2+...+9 = 45
    assert!(output.contains("45"), "got: {}", output);
}

// ============================================================================
// Channel Communication
// ============================================================================

#[test]
fn test_channel_basic_send_recv() {
    let code = r#"
def producer(c):
    send(c, 42)

def consumer(c):
    val = recv(c)
    print(val)

def test():
    c = chan(1)
    sync:
        task producer(c)
        task consumer(c)
test()
"#;
    let output = run_viper_code(code).expect("channel send/recv should work");
    assert!(output.contains("42"), "got: {}", output);
}

#[test]
fn test_channel_multiple_messages() {
    let code = r#"
received = []

def producer(c):
    send(c, 1)
    send(c, 2)
    send(c, 3)

def consumer(c):
    a = recv(c)
    b = recv(c)
    d = recv(c)
    received.append(a)
    received.append(b)
    received.append(d)

def test():
    c = chan(10)
    sync:
        task producer(c)
        task consumer(c)
    received.sort()
    print(received[0])
    print(received[2])
test()
"#;
    let output = run_viper_code(code).expect("multiple channel messages should work");
    assert!(output.contains("1"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_channel_pipeline() {
    let code = r#"
def stage1(out_c):
    for i in range(5):
        send(out_c, i)

def stage2(in_c, out_c):
    for _ in range(5):
        val = recv(in_c)
        send(out_c, val * 2)

result = []

def stage3(in_c):
    for _ in range(5):
        val = recv(in_c)
        result.append(val)

def test():
    c1 = chan(10)
    c2 = chan(10)
    sync:
        task stage1(c1)
        task stage2(c1, c2)
        task stage3(c2)
    result.sort()
    print(len(result))
    print(result[4])
test()
"#;
    let output = run_viper_code(code).expect("pipeline should work");
    assert!(output.contains("5"), "got: {}", output);
    assert!(output.contains("8"), "got: {}", output);
}

// ============================================================================
// WaitGroup
// ============================================================================

#[test]
fn test_waitgroup_basic() {
    let code = r#"
done_count = 0

def worker(wg):
    global done_count
    done_count += 1
    done(wg)

def test():
    wg = WaitGroup()
    add(wg, 3)
    task worker(wg)
    task worker(wg)
    task worker(wg)
    wait(wg)
    print(done_count)
test()
"#;
    let output = run_viper_code(code).expect("WaitGroup should work");
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// Global Atomic Increment (Data Race Safety)
// ============================================================================

#[test]
fn test_atomic_global_increment() {
    let code = r#"
counter = 0

def bump():
    global counter
    counter += 1

def test():
    global counter
    sync:
        task bump()
        task bump()
        task bump()
        task bump()
        task bump()
    print(counter)
test()
"#;
    let output = run_viper_code(code).expect("atomic global increment should work");
    assert!(output.contains("5"), "got: {}", output);
}
