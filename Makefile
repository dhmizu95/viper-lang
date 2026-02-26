.PHONY: build test lint fmt run clean bench dev check aot

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

# Run benchmarks
bench:
	cargo run -- bench

# Helper for AOT compilation
aot: build runtime
	@echo "AOT compilation ready. Use 'viper build <file>'"
