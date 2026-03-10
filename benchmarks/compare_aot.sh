#!/bin/bash
# Compare AOT performance across languages

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VIPER_BIN="$PROJECT_ROOT/target/release/viper"

echo "=== AOT Cross-Language Performance Comparison ==="
echo

# Compile all versions
echo "Compiling all languages..."

# Viper AOT -O2
for bench in 01_fibonacci 02_prime_sieve 03_matrix_mul; do
    echo -n "  Viper AOT -O2 ($bench)... "
    "$VIPER_BIN" build -O2 "$SCRIPT_DIR/viper/${bench}.vp" -o "/tmp/viper_${bench}" 2>&1 | tail -1
    cp "/tmp/viper_${bench}_bin" "/tmp/viper_${bench}_aot" 2>/dev/null || true
done

# C -O3
echo -n "  C -O3... "
gcc -O3 -march=native -flto -o /tmp/c_fib "$SCRIPT_DIR/c/01_fibonacci.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_sieve "$SCRIPT_DIR/c/02_prime_sieve.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_matrix "$SCRIPT_DIR/c/03_matrix_mul.c" 2>/dev/null
echo "done"

# Rust -O3
echo -n "  Rust -O3... "
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_fib "$SCRIPT_DIR/rust/01_fibonacci.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_sieve "$SCRIPT_DIR/rust/02_prime_sieve.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_matrix "$SCRIPT_DIR/rust/03_matrix_mul.rs" 2>/dev/null
echo "done"

# Go
echo -n "  Go... "
go build -ldflags="-s -w" -o /tmp/go_fib "$SCRIPT_DIR/go/01_fibonacci.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_sieve "$SCRIPT_DIR/go/02_prime_sieve.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_matrix "$SCRIPT_DIR/go/03_matrix_mul.go" 2>/dev/null
echo "done"

echo
echo "Running benchmarks (5 iterations each)..."
echo

run_bench() {
    local binary=$1
    local runs=5
    
    total_ns=0
    for i in $(seq 1 $runs); do
        start=$(date +%s%N)
        "$binary" > /dev/null 2>&1
        end=$(date +%s%N)
        elapsed=$((end - start))
        total_ns=$((total_ns + elapsed))
    done
    avg_ns=$((total_ns / runs))
    avg_ms=$((avg_ns / 1000000))
    echo "$avg_ms"
}

echo "--- Fibonacci (n=35) ---"
printf "%-15s %12s\n" "Language" "Avg Time (ms)"
printf "%-15s %12s\n" "--------" "-------------"
c_time=$(run_bench /tmp/c_fib)
rust_time=$(run_bench /tmp/rust_fib)
go_time=$(run_bench /tmp/go_fib)
viper_time=$(run_bench /tmp/viper_01_fibonacci_aot)
printf "%-15s %12s\n" "C -O3" "$c_time"
printf "%-15s %12s\n" "Rust -O3" "$rust_time"
printf "%-15s %12s\n" "Go" "$go_time"
printf "%-15s %12s\n" "Viper AOT -O2" "$viper_time"

echo
echo "--- Prime Sieve (n=5000) ---"
printf "%-15s %12s\n" "Language" "Avg Time (ms)"
printf "%-15s %12s\n" "--------" "-------------"
c_time=$(run_bench /tmp/c_sieve)
rust_time=$(run_bench /tmp/rust_sieve)
go_time=$(run_bench /tmp/go_sieve)
viper_time=$(run_bench /tmp/viper_02_prime_sieve_aot)
printf "%-15s %12s\n" "C -O3" "$c_time"
printf "%-15s %12s\n" "Rust -O3" "$rust_time"
printf "%-15s %12s\n" "Go" "$go_time"
printf "%-15s %12s\n" "Viper AOT -O2" "$viper_time"

echo
echo "--- Matrix Mul (30x30) ---"
printf "%-15s %12s\n" "Language" "Avg Time (ms)"
printf "%-15s %12s\n" "--------" "-------------"
c_time=$(run_bench /tmp/c_matrix)
rust_time=$(run_bench /tmp/rust_matrix)
go_time=$(run_bench /tmp/go_matrix)
viper_time=$(run_bench /tmp/viper_03_matrix_mul_aot)
printf "%-15s %12s\n" "C -O3" "$c_time"
printf "%-15s %12s\n" "Rust -O3" "$rust_time"
printf "%-15s %12s\n" "Go" "$go_time"
printf "%-15s %12s\n" "Viper AOT -O2" "$viper_time"

# Cleanup
rm -f /tmp/c_* /tmp/rust_* /tmp/go_* /tmp/viper_* /tmp/*.o /tmp/*.bc

echo
echo "=== Comparison Complete ==="
