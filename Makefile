.PHONY: build test lint fmt run clean bench dev check aot
.PHONY: bench-safe bench-safe-one bench-safe-fib bench-safe-quicksort
.PHONY: pgo pgo-clean pgo-bench

# Default target: build the compiler
build:
	cargo build

# Run the compiler in JIT mode with -O3 optimization
run:
	cargo run -- run

# Run with -O0 for debugging compiler output
dev:
	cargo run -- run -O0

# Check syntax and types without full build
check:
	cargo check

# Run tests
test:
	cargo test

# Run linter
lint:
	cargo run -- lint

# Format code
fmt:
	cargo fmt

# Build the runtime library
runtime:
	cd runtime && make

# Clean build artifacts
clean:
	cargo clean
	cd runtime && make clean

# Run Viper internal benchmarks
bench:
	cargo run --bin viper -- bench

# Test AOT compilation (known issue: linking fails)
bench-aot-test:
	cd benchmarks && ./test_aot.sh

# Run benchmarks with crash protection (safe mode)
bench-safe:
	cd benchmarks && ./benchmark_runner.sh all

# Run single benchmark with safe mode (1 iteration, quick test)
bench-safe-one:
	cd benchmarks && ./benchmark_runner.sh -i 1 01_fibonacci

# Run Fibonacci with safe mode
bench-safe-fibonacci:
	cd benchmarks && ./benchmark_runner.sh 01_fibonacci

# Run QuickSort with safe mode
bench-safe-quicksort:
	cd benchmarks && ./benchmark_runner.sh 04_quicksort

# Helper for AOT compilation
aot: build runtime
	@echo "AOT compilation ready. Use 'viper build <file>'"

# PGO (Profile-Guided Optimization) targets
# PGO improves performance by 10-30% by optimizing for typical workloads

# Clean PGO data
pgo-clean:
	rm -rf target/pgo-data
	mkdir -p target/pgo-data
	@echo "PGO data directory cleaned"

# Build PGO-instrumented compiler
pgo-instrument: pgo-clean
	LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" cargo build --profile pgo-instrument
	@echo "PGO-instrumented compiler built"

# Run benchmarks with instrumented compiler to collect profiles
pgo-run: pgo-instrument
	@echo "Running instrumented compiler on benchmarks..."
	LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" cargo run --profile pgo-instrument -- run benchmarks/viper/*.vp
	@echo "Profile data collected"

# Merge PGO profiles
pgo-merge:
	@echo "Merging PGO profiles..."
	llvm-profdata merge -sparse target/pgo-data/*.profraw -o target/pgo-data/merged.profdata
	@echo "Profiles merged to target/pgo-data/merged.profdata"

# Build PGO-optimized compiler
pgo: pgo-run pgo-merge
	RUSTFLAGS="-Cprofile-use=target/pgo-data/merged.profdata" cargo build --profile pgo
	@echo "PGO-optimized compiler built successfully"
	@echo "Binary: target/pgo/viper"

# Quick PGO build (skip instrumented run, use existing profiles)
pgo-quick:
	RUSTFLAGS="-Cprofile-use=target/pgo-data/merged.profdata" cargo build --profile pgo
	@echo "PGO-optimized compiler built (using existing profiles)"

# Benchmark PGO vs regular release
pgo-bench: pgo
	@echo "Running benchmarks with PGO-optimized compiler..."
	./target/pgo/viper bench
	@echo "PGO benchmark complete"
