# Profile-Guided Optimization (PGO) for Viper

## Overview

Profile-Guided Optimization (PGO) can improve Viper compiler performance by 10-30% by optimizing the compiler binary based on real-world usage patterns.

## How PGO Works

1. **Instrumentation Phase**: Build the compiler with profiling instrumentation
2. **Profile Collection**: Run the instrumented compiler on typical workloads
3. **Profile Merging**: Combine collected profiles into a single data file
4. **Optimized Build**: Rebuild the compiler using the profile data

## Quick Start

### Full PGO Build (Recommended)

```bash
# Clean any existing PGO data
make pgo-clean

# Build instrumented compiler, run benchmarks, and create optimized binary
make pgo
```

This will:
1. Build an instrumented version of the compiler
2. Run it on all Viper benchmarks
3. Merge the collected profiles
4. Build the final PGO-optimized compiler

The optimized binary will be at `target/pgo/viper`.

### Quick PGO Build (Using Existing Profiles)

If you already have profile data:

```bash
make pgo-quick
```

### Manual PGO Build

```bash
# Step 1: Clean PGO data directory
make pgo-clean

# Step 2: Build instrumented compiler
make pgo-instrument

# Step 3: Run your workloads
LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" ./target/pgo-instrument/viper run your_program.vp

# Step 4: Merge profiles
make pgo-merge

# Step 5: Build optimized compiler
RUSTFLAGS="-Cprofile-use=target/pgo-data/merged.profdata" cargo build --profile pgo
```

## Make Targets

| Target | Description |
|--------|-------------|
| `make pgo-clean` | Clean PGO data directory |
| `make pgo-instrument` | Build PGO-instrumented compiler |
| `make pgo-run` | Run instrumented compiler on benchmarks |
| `make pgo-merge` | Merge collected profiles |
| `make pgo` | Full PGO build (instrument + run + merge + build) |
| `make pgo-quick` | Quick build using existing profiles |
| `make pgo-bench` | Run benchmarks with PGO-optimized compiler |

## Performance Expectations

Typical performance improvements from PGO:

- **Compilation speed**: 10-15% faster
- **Generated code quality**: 5-10% better runtime performance
- **Overall throughput**: 15-25% improvement

Actual improvements depend on:
- How representative your training workloads are
- The specific optimization opportunities in your codebase
- The LLVM version and optimization passes used

## Profile Data Location

- Raw profiles: `target/pgo-data/viper-*.profraw`
- Merged profile: `target/pgo-data/merged.profdata`

## Troubleshooting

### "No profile data found"

Make sure to run the instrumented compiler on some workloads before building the optimized version:

```bash
make pgo-run
```

### "Profile data mismatch"

If you change the compiler source code significantly, old profiles may not apply. Clean and regenerate:

```bash
make pgo-clean
make pgo
```

### Build fails with PGO

Try building without PGO first to ensure the base build works:

```bash
cargo build --release
```

## Advanced Usage

### Custom Training Workloads

Instead of using the default benchmarks, you can train on your own workloads:

```bash
# Build instrumented compiler
make pgo-instrument

# Run on your typical workloads
LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" ./target/pgo-instrument/viper run project1/*.vp
LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" ./target/pgo-instrument/viper run project2/*.vp

# Merge and build
make pgo-merge
RUSTFLAGS="-Cprofile-use=target/pgo-data/merged.profdata" cargo build --profile pgo
```

### Thin LTO with PGO

For faster builds with PGO:

```bash
RUSTFLAGS="-Cprofile-use=target/pgo-data/merged.profdata -Clto=thin" cargo build --profile pgo
```

## References

- [Rust PGO Documentation](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)
- [LLVM PGO Documentation](https://llvm.org/docs/ProposedPGO.html)
