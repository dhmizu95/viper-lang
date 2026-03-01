# Insert 1M Integers - Cross-Language Benchmark Results

## Results Summary

| Language | Method | User Time | System Time | Total (real) |
|----------|--------|-----------|-------------|--------------|
| **C** | append | 0.000s | 0.004s | **0.006s** |
| **C** | prealloc | 0.000s | 0.006s | **0.006s** |
| **Rust** | append | 0.000s | 0.006s | **0.006s** |
| **Rust** | prealloc | 0.000s | 0.010s | **0.009s** |
| **Go** | append | 0.032s | 0.056s | **0.047s** |
| **Go** | prealloc | 0.004s | 0.011s | **0.013s** |
| **Viper** | append | 0.165s | 0.157s | **0.303s** |

## Performance Comparison (relative to C)

| Language | Speed vs C | Notes |
|----------|------------|-------|
| C | 1.0x (baseline) | Native compiled, -O3 |
| Rust | ~1.0x | Native compiled, -O |
| Go | ~2-8x slower | GC + runtime overhead |
| Viper | ~50x slower | **JIT compilation + interpreter overhead** |

## Key Observations

### C/Rust
- Near-instant execution (<10ms)
- Direct memory management
- No runtime overhead
- Pre-allocation shows minimal benefit for simple int insertion

### Go
- Append: 47ms (GC pressure from growing slice)
- Pre-alloc: 13ms (**3.6x faster** with pre-allocation)
- GC and bounds checking add overhead

### Viper
- Total: 303ms (includes JIT compilation)
- **JIT startup cost** is significant portion
- ARC (reference counting) adds overhead
- Dynamic list with bounds checking
- Python-like safety features

## Optimization Opportunities for Viper

1. **AOT compilation** - Remove JIT startup cost
2. **Pre-allocation syntax** - Add `[0] * N` optimization
3. **ARC optimizations** - Batch retain/release
4. **Inline list operations** - Reduce function call overhead

## Benchmark Files

```
benchmark/
├── insert_1m_integers.vp    # Viper (append)
├── c/
│   ├── insert_1m_append.c
│   └── insert_1m_prealloc.c
├── rust/
│   ├── insert_1m_append.rs
│   └── insert_1m_prealloc.rs
└── go/
    ├── insert_1m_append.go
    └── insert_1m_prealloc.go
```

## Run Your Own

```bash
# Viper
cargo run --release -- run benchmark/insert_1m_integers.vp -O 3

# C
gcc -O3 benchmark/c/insert_1m_append.c -o c_append && ./c_append

# Rust
rustc -O benchmark/rust/insert_1m_append.rs -o rust_append && ./rust_append

# Go
go build benchmark/go/insert_1m_append.go -o go_append && ./go_append
```
