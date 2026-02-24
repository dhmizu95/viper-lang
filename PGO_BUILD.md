# LTO and PGO Build Guide

This document describes how to use Link-Time Optimization (LTO) and Profile-Guided Optimization (PGO) for the Viper compiler.

## Overview

- **LTO (Link-Time Optimization)**: Enables whole-program optimization across crate boundaries, potentially improving performance by 10-20%.
- **PGO (Profile-Guided Optimization)**: Uses runtime profiling data to guide optimization decisions, potentially improving performance by an additional 5-15%.

## Build Profiles

### Standard Release (with LTO)
```bash
cargo build --release
```
This builds with fat LTO enabled for maximum optimization.

### Thin LTO (Faster Builds)
```bash
cargo build --profile release-thin
```
Uses thin LTO for faster builds with slightly less optimization.

### PGO Build Cycle

For maximum performance, use the PGO build script:

```bash
# Full PGO cycle (recommended)
./scripts/pgo.sh all

# Or step by step:
./scripts/pgo.sh instrument   # Build instrumented binary
./scripts/pgo.sh run          # Run workloads to collect profiles
./scripts/pgo.sh merge        # Merge profile data
./scripts/pgo.sh build        # Build final optimized binary
```

### Manual PGO Build

```bash
# 1. Build instrumented binary
LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" \
    cargo build --profile pgo-instrument --bin viper

# 2. Run your workloads
./target/pgo-instrument/viper <your-workload>

# 3. Merge profile data
llvm-profdata merge -sparse target/pgo-data/*.profraw -o target/pgo-data/merged.profdata

# 4. Build with profile data
RUSTFLAGS="-Cprofile-use=target/pgo-data/merged.profdata" \
    cargo build --profile pgo --bin viper
```

## Profile Comparison

| Profile | LTO | PGO | Build Time | Performance |
|---------|-----|-----|------------|-------------|
| `release` | Fat | No | Medium | Good |
| `release-thin` | Thin | No | Fast | Good |
| `pgo-instrument` | No | Instrument | Fast | Baseline |
| `pgo` | Fat | Yes | Slow | Best |

## Cleaning PGO Data

```bash
# Clean PGO data
./scripts/pgo.sh clean

# Or manually
rm -rf target/pgo-data target/pgo-instrument target/pgo target/pgo-output
```

## Requirements

- LLVM 20 (for `llvm-profdata`)
- Rust nightly (recommended for latest optimizations)

## Troubleshooting

### llvm-profdata not found
Install LLVM tools:
```bash
# Ubuntu/Debian
sudo apt install llvm-20

# macOS
brew install llvm
```

### PGO profile mismatch errors
Clean and rebuild:
```bash
./scripts/pgo.sh clean
cargo clean
./scripts/pgo.sh all
```

## Performance Testing

Compare binary sizes:
```bash
ls -lh target/release/viper target/pgo/viper
```

Benchmark your workloads with each build to measure improvements.
