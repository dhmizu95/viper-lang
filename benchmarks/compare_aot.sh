#!/bin/bash
# Compare AOT performance across languages and optimization levels

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VIPER_BIN="$PROJECT_ROOT/target/release/viper"
ITERATIONS=${ITERATIONS:-5}

echo "=== AOT Cross-Language Performance Comparison (All Opt Levels) ==="
echo

# Compile all versions
echo "Compiling all languages..."

# Viper AOT -O1, -O2, -O3
for opt_level in 1 2 3; do
    for bench in 01_fibonacci 02_prime_sieve 03_matrix_mul 04_quicksort 05_matrix_mul 06_prime_sieve 07_string_ops; do
        echo -n "  Viper AOT -O${opt_level} ($bench)... "
        "$VIPER_BIN" build -O${opt_level} "$SCRIPT_DIR/viper/${bench}.vp" -o "/tmp/viper_o${opt_level}_${bench}" 2>&1 | tail -1
    done
done

# C -O3
echo -n "  C -O3... "
gcc -O3 -march=native -flto -o /tmp/c_fib "$SCRIPT_DIR/c/01_fibonacci.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_sieve "$SCRIPT_DIR/c/02_prime_sieve.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_matrix "$SCRIPT_DIR/c/03_matrix_mul.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_quicksort "$SCRIPT_DIR/c/04_quicksort.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_matrix2 "$SCRIPT_DIR/c/05_matrix_mul.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_sieve2 "$SCRIPT_DIR/c/06_prime_sieve.c" 2>/dev/null
gcc -O3 -march=native -flto -o /tmp/c_string "$SCRIPT_DIR/c/07_string_ops.c" 2>/dev/null
echo "done"

# Rust -O3
echo -n "  Rust -O3... "
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_fib "$SCRIPT_DIR/rust/01_fibonacci.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_sieve "$SCRIPT_DIR/rust/02_prime_sieve.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_matrix "$SCRIPT_DIR/rust/03_matrix_mul.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_quicksort "$SCRIPT_DIR/rust/04_quicksort.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_matrix2 "$SCRIPT_DIR/rust/05_matrix_mul.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_sieve2 "$SCRIPT_DIR/rust/06_prime_sieve.rs" 2>/dev/null
rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o /tmp/rust_string "$SCRIPT_DIR/rust/07_string_ops.rs" 2>/dev/null
echo "done"

# Go
echo -n "  Go... "
go build -ldflags="-s -w" -o /tmp/go_fib "$SCRIPT_DIR/go/01_fibonacci.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_sieve "$SCRIPT_DIR/go/02_prime_sieve.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_matrix "$SCRIPT_DIR/go/03_matrix_mul.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_quicksort "$SCRIPT_DIR/go/04_quicksort.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_matrix2 "$SCRIPT_DIR/go/05_matrix_mul.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_sieve2 "$SCRIPT_DIR/go/06_prime_sieve.go" 2>/dev/null
go build -ldflags="-s -w" -o /tmp/go_string "$SCRIPT_DIR/go/07_string_ops.go" 2>/dev/null
echo "done"

echo
echo "Running benchmarks ($ITERATIONS iterations each)..."
echo

run_bench() {
    local binary=$1
    local runs=$ITERATIONS

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

print_header() {
    printf "\n%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
        "Benchmark" "Viper JIT" "O1" "O2" "O3" "C -O3" "Rust -O3" "Go"
    printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
        "--------------------" "----------" "----------" "----------" "----------" "----------" "----------" "----------"
}

# Compile Viper JIT (no separate binary, just measure run time)
run_viper_jit() {
    local name=$1
    local runs=$ITERATIONS
    total_ns=0
    for i in $(seq 1 $runs); do
        start=$(date +%s%N)
        "$VIPER_BIN" run -O3 "$SCRIPT_DIR/viper/${name}.vp" > /dev/null 2>&1
        end=$(date +%s%N)
        elapsed=$((end - start))
        total_ns=$((total_ns + elapsed))
    done
    avg_ns=$((total_ns / runs))
    avg_ms=$((avg_ns / 1000000))
    echo "$avg_ms"
}

echo
print_header

# Fibonacci
c_time=$(run_bench /tmp/c_fib)
rust_time=$(run_bench /tmp/rust_fib)
go_time=$(run_bench /tmp/go_fib)
viper_o1=$(run_bench /tmp/viper_o1_01_fibonacci_bin)
viper_o2=$(run_bench /tmp/viper_o2_01_fibonacci_bin)
viper_o3=$(run_bench /tmp/viper_o3_01_fibonacci_bin)
viper_jit=$(run_viper_jit 01_fibonacci)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "01_fibonacci" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# Prime Sieve
c_time=$(run_bench /tmp/c_sieve)
rust_time=$(run_bench /tmp/rust_sieve)
go_time=$(run_bench /tmp/go_sieve)
viper_o1=$(run_bench /tmp/viper_o1_02_prime_sieve_bin)
viper_o2=$(run_bench /tmp/viper_o2_02_prime_sieve_bin)
viper_o3=$(run_bench /tmp/viper_o3_02_prime_sieve_bin)
viper_jit=$(run_viper_jit 02_prime_sieve)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "02_prime_sieve" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# Matrix Mul
c_time=$(run_bench /tmp/c_matrix)
rust_time=$(run_bench /tmp/rust_matrix)
go_time=$(run_bench /tmp/go_matrix)
viper_o1=$(run_bench /tmp/viper_o1_03_matrix_mul_bin)
viper_o2=$(run_bench /tmp/viper_o2_03_matrix_mul_bin)
viper_o3=$(run_bench /tmp/viper_o3_03_matrix_mul_bin)
viper_jit=$(run_viper_jit 03_matrix_mul)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "03_matrix_mul" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# QuickSort
c_time=$(run_bench /tmp/c_quicksort)
rust_time=$(run_bench /tmp/rust_quicksort)
go_time=$(run_bench /tmp/go_quicksort)
viper_o1=$(run_bench /tmp/viper_o1_04_quicksort_bin)
viper_o2=$(run_bench /tmp/viper_o2_04_quicksort_bin)
viper_o3=$(run_bench /tmp/viper_o3_04_quicksort_bin)
viper_jit=$(run_viper_jit 04_quicksort)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "04_quicksort" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# Matrix Mul Array
c_time=$(run_bench /tmp/c_matrix2)
rust_time=$(run_bench /tmp/rust_matrix2)
go_time=$(run_bench /tmp/go_matrix2)
viper_o1=$(run_bench /tmp/viper_o1_05_matrix_mul_bin)
viper_o2=$(run_bench /tmp/viper_o2_05_matrix_mul_bin)
viper_o3=$(run_bench /tmp/viper_o3_05_matrix_mul_bin)
viper_jit=$(run_viper_jit 05_matrix_mul)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "05_matrix_mul_array" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# Prime Sieve Array
c_time=$(run_bench /tmp/c_sieve2)
rust_time=$(run_bench /tmp/rust_sieve2)
go_time=$(run_bench /tmp/go_sieve2)
viper_o1=$(run_bench /tmp/viper_o1_06_prime_sieve_bin)
viper_o2=$(run_bench /tmp/viper_o2_06_prime_sieve_bin)
viper_o3=$(run_bench /tmp/viper_o3_06_prime_sieve_bin)
viper_jit=$(run_viper_jit 06_prime_sieve)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "06_prime_sieve_array" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# String Operations
c_time=$(run_bench /tmp/c_string)
rust_time=$(run_bench /tmp/rust_string)
go_time=$(run_bench /tmp/go_string)
viper_o1=$(run_bench /tmp/viper_o1_07_string_ops_bin)
viper_o2=$(run_bench /tmp/viper_o2_07_string_ops_bin)
viper_o3=$(run_bench /tmp/viper_o3_07_string_ops_bin)
viper_jit=$(run_viper_jit 07_string_ops)
printf "%-20s %10s %10s %10s %10s %10s %10s %10s\n" \
    "07_string_ops" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time"

# Cleanup
rm -f /tmp/c_* /tmp/rust_* /tmp/go_* /tmp/viper_* /tmp/*.o /tmp/*.bc

echo
echo "=== Comparison Complete ==="
