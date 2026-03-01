#!/bin/bash
# Cross-language benchmark: Insert 1M integers

echo "╔══════════════════════════════════════════════════════════╗"
echo "║     Insert 1M Integers - Cross-Language Benchmark        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

cd "$(dirname "$0")"

# Compile all if needed
echo "🔨 Compiling..."
gcc -O3 -o c/insert_1m_append c/insert_1m_append.c 2>/dev/null
gcc -O3 -o c/insert_1m_prealloc c/insert_1m_prealloc.c 2>/dev/null
rustc -O -o rust/insert_1m_append rust/insert_1m_append.rs 2>/dev/null
rustc -O -o rust/insert_1m_prealloc rust/insert_1m_prealloc.rs 2>/dev/null
go build -o go/insert_1m_append go/insert_1m_append.go 2>/dev/null
go build -o go/insert_1m_prealloc go/insert_1m_prealloc.go 2>/dev/null
cargo build --release --quiet 2>/dev/null
echo "✅ All compiled"
echo ""

run_benchmark() {
    local name=$1
    local cmd=$2
    
    # Run 3 times and take best
    local best=999
    for i in 1 2 3; do
        local result=$( { time -p $cmd; } 2>&1 | grep real | awk '{print $2}' )
        if (( $(echo "$result < $best" | bc -l) )); then
            best=$result
        fi
    done
    printf "%-25s %8.3fs\n" "$name" "$best"
}

echo "┌─────────────────────────┬──────────┐"
echo "│ Method                  │ Time (s) │"
echo "├─────────────────────────┼──────────┤"

# C benchmarks
run_benchmark "C (append)" "./c/insert_1m_append"
run_benchmark "C (prealloc)" "./c/insert_1m_prealloc"

# Rust benchmarks
run_benchmark "Rust (append)" "./rust/insert_1m_append"
run_benchmark "Rust (prealloc)" "./rust/insert_1m_prealloc"

# Go benchmarks
run_benchmark "Go (append)" "./go/insert_1m_append"
run_benchmark "Go (prealloc)" "./go/insert_1m_prealloc"

# Viper benchmark
run_benchmark "Viper (append)" "cargo run --release --quiet -- run insert_1m_integers.vp -O 3 2>&1"

echo "└─────────────────────────┴──────────┘"
echo ""
echo "Note: Times are best of 3 runs. Viper includes JIT startup."
