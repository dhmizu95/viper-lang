#!/bin/bash
# Compare AOT performance across languages and optimization levels

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VIPER_BIN="$PROJECT_ROOT/target/release/viper"
ITERATIONS=${ITERATIONS:-5}
TMPDIR="${TMPDIR:-/tmp}"
BENCHMARKS=(
    "01_fibonacci"
    "02_prime_sieve"
    "03_matrix_mul"
    "04_quicksort"
    "05_matrix_mul"
    "06_prime_sieve"
    "07_string_ops"
    "08_int_hotloop"
    "09_i64_hotloop"
    "10_function_calls"
    "11_string_concat_scan"
    "12_bigint_overflow"
)

if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="python3"
elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN="python"
else
    PYTHON_BIN=""
fi

echo "=== AOT Cross-Language Performance Comparison (All Opt Levels) ==="
echo

# Compile all versions
echo "Compiling all languages..."

# Viper AOT -O1, -O2, -O3
for opt_level in 1 2 3; do
    for bench in "${BENCHMARKS[@]}"; do
        echo -n "  Viper AOT -O${opt_level} ($bench)... "
        "$VIPER_BIN" build -O${opt_level} "$SCRIPT_DIR/viper/${bench}.vp" -o "$TMPDIR/viper_o${opt_level}_${bench}" 2>&1 | tail -1
    done
done

for bench in "${BENCHMARKS[@]}"; do
    echo -n "  C -O3 ($bench)... "
    gcc -O3 -march=native -flto -o "$TMPDIR/c_${bench}" "$SCRIPT_DIR/c/${bench}.c" 2>/dev/null && echo "done" || echo "skip"
    echo -n "  Rust -O3 ($bench)... "
    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$TMPDIR/rust_${bench}" "$SCRIPT_DIR/rust/${bench}.rs" 2>/dev/null && echo "done" || echo "skip"
    echo -n "  Go ($bench)... "
    go build -ldflags="-s -w" -o "$TMPDIR/go_${bench}" "$SCRIPT_DIR/go/${bench}.go" 2>/dev/null && echo "done" || echo "skip"
done

echo
echo "Running benchmarks ($ITERATIONS iterations each)..."
echo

run_bench() {
    local cmd=("$@")
    total_ns=0
    for _ in $(seq 1 "$ITERATIONS"); do
        start=$(date +%s%N)
        "${cmd[@]}" > /dev/null 2>&1
        end=$(date +%s%N)
        total_ns=$((total_ns + end - start))
    done
    echo $(((total_ns / ITERATIONS) / 1000000))
}

print_header() {
    printf "\n%-20s %10s %10s %10s %10s %10s %10s %10s %10s\n" \
        "Benchmark" "Viper JIT" "O1" "O2" "O3" "C -O3" "Rust -O3" "Go" "Python"
    printf "%-20s %10s %10s %10s %10s %10s %10s %10s %10s\n" \
        "--------------------" "----------" "----------" "----------" "----------" "----------" "----------" "----------" "----------"
}

# Compile Viper JIT (no separate binary, just measure run time)
run_viper_jit() {
    local name=$1
    run_bench "$VIPER_BIN" run -O3 "$SCRIPT_DIR/viper/${name}.vp"
}

echo
print_header

for bench in "${BENCHMARKS[@]}"; do
    c_time=$(run_bench "$TMPDIR/c_${bench}")
    rust_time=$(run_bench "$TMPDIR/rust_${bench}")
    go_time=$(run_bench "$TMPDIR/go_${bench}")
    viper_o1=$(run_bench "$TMPDIR/viper_o1_${bench}_bin")
    viper_o2=$(run_bench "$TMPDIR/viper_o2_${bench}_bin")
    viper_o3=$(run_bench "$TMPDIR/viper_o3_${bench}_bin")
    viper_jit=$(run_viper_jit "$bench")
    py_time="N/A"
    if [ -n "$PYTHON_BIN" ] && [ -f "$SCRIPT_DIR/python/${bench}.py" ]; then
        py_time=$(run_bench "$PYTHON_BIN" "$SCRIPT_DIR/python/${bench}.py")
    fi
    printf "%-20s %10s %10s %10s %10s %10s %10s %10s %10s\n" \
        "$bench" "$viper_jit" "$viper_o1" "$viper_o2" "$viper_o3" "$c_time" "$rust_time" "$go_time" "$py_time"
done

# Cleanup
rm -f "$TMPDIR"/c_* "$TMPDIR"/rust_* "$TMPDIR"/go_* "$TMPDIR"/viper_*

echo
echo "=== Comparison Complete ==="
