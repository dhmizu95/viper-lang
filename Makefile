.PHONY: build test lint fmt run clean bench dev check aot
.PHONY: bench-all bench-fibonacci bench-quicksort bench-compare
.PHONY: bench-aot-test bench-opt-compare bench-aot-compare
.PHONY: bench-safe bench-safe-one bench-safe-fib bench-safe-sort

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
	cargo run -- bench

# Run cross-language benchmarks (all, JIT mode)
bench-all:
	cd benchmarks && ./runner.sh all

# Run Fibonacci benchmark only
bench-fibonacci:
	cd benchmarks && ./runner.sh 01_fibonacci

# Run QuickSort benchmark only
bench-quicksort:
	cd benchmarks && ./runner.sh 02_quicksort

# Run all benchmarks with comparison output (10 iterations)
bench-compare:
	cd benchmarks && ./runner.sh -i 10 all

# Run all optimization levels comparison (JIT, O1, O2, O3 + C/Rust/Go)
bench-opt-compare:
	cd benchmarks && ./runner.sh --opt-compare all

# Run detailed AOT comparison table (all opt levels + C/Rust/Go)
bench-aot-compare:
	cd benchmarks && ./compare_aot.sh

# Test AOT compilation (known issue: linking fails)
bench-aot-test:
	cd benchmarks && ./test_aot.sh

# Run benchmarks with crash protection (safe mode)
bench-safe:
	cd benchmarks && ./safe_runner.sh all

# Run single benchmark with safe mode (1 iteration, quick test)
bench-safe-one:
	cd benchmarks && ./safe_runner.sh -i 1 01_fibonacci

# Run Fibonacci with safe mode
bench-safe-fibonacci:
	cd benchmarks && ./safe_runner.sh 01_fibonacci

# Run QuickSort with safe mode
bench-safe-quicksort:
	cd benchmarks && ./safe_runner.sh 04_quicksort

# Helper for AOT compilation
aot: build runtime
	@echo "AOT compilation ready. Use 'viper build <file>'"
