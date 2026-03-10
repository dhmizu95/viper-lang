#!/bin/bash
# Cross-Language Benchmark Runner
# Compares Viper JIT, Viper AOT (O1/O2/O3), C, Rust, and Go performance

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ITERATIONS=${ITERATIONS:-3}
VIPER_BIN="$PROJECT_ROOT/target/release/viper"
TMPDIR="${TMPDIR:-/tmp}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Benchmark list
BENCHMARKS=(
    "01_fibonacci"
    "02_prime_sieve"
    "03_matrix_mul"
    "04_quicksort"
    "05_matrix_mul"
    "06_prime_sieve"
    "07_string_ops"
)

# Check prerequisites
check_prereqs() {
    echo -e "${BLUE}Checking prerequisites...${NC}"

    if ! command -v gcc &> /dev/null; then
        echo -e "${RED}Error: gcc not found${NC}"
        exit 1
    fi

    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}Error: rustc not found${NC}"
        exit 1
    fi

    if ! command -v go &> /dev/null; then
        echo -e "${RED}Error: go not found${NC}"
        exit 1
    fi

    if [ ! -f "$VIPER_BIN" ]; then
        echo -e "${YELLOW}Warning: Viper binary not found. Building...${NC}"
        cd "$PROJECT_ROOT" && cargo build --release
    fi

    echo -e "${GREEN}All prerequisites met${NC}"
    echo
}

# Time a benchmark run
run_bench() {
    local cmd=$1
    local runs=$2

    total_ns=0
    for i in $(seq 1 $runs); do
        start=$(date +%s%N)
        eval "$cmd" > /dev/null 2>&1
        end=$(date +%s%N)
        elapsed=$((end - start))
        total_ns=$((total_ns + elapsed))
    done
    avg_ns=$((total_ns / runs))
    avg_ms=$((avg_ns / 1000000))
    echo "$avg_ms"
}

# Run single benchmark with all optimization levels and languages
run_benchmark_full() {
    local name=$1
    local file="$SCRIPT_DIR/viper/${name}.vp"
    local c_file="$SCRIPT_DIR/c/${name}.c"
    local rust_file="$SCRIPT_DIR/rust/${name}.rs"
    local go_file="$SCRIPT_DIR/go/${name}.go"

    echo -e "${MAGENTA}========================================${NC}"
    echo -e "${MAGENTA}Benchmark: $name${NC}"
    echo -e "${MAGENTA}========================================${NC}"
    echo

    # Verify output (run once)
    echo "  Output verification:"
    if [ -f "$c_file" ]; then
        gcc -O3 -march=native -flto -o "$TMPDIR/bench_c_out_$$" "$c_file" 2>/dev/null
        echo -n "    C:       "
        "$TMPDIR/bench_c_out_$$" 2>&1 | tail -1
        rm -f "$TMPDIR/bench_c_out_$$"
    fi

    echo
    echo "  Timing (avg of $ITERATIONS runs):"

    # Viper JIT
    viper_jit_time="N/A"
    if [ -f "$file" ]; then
        viper_jit_time=$(run_bench "$VIPER_BIN run -O3 $file" "$ITERATIONS" 2>/dev/null || echo "N/A")
    fi

    # Viper AOT -O1
    viper_o1_time="N/A"
    if [ -f "$file" ]; then
        "$VIPER_BIN" build -O1 "$file" -o "$TMPDIR/bench_viper_o1_$$" 2>/dev/null
        viper_o1_time=$(run_bench "$TMPDIR/bench_viper_o1_$$_bin" "$ITERATIONS")
        rm -f "$TMPDIR/bench_viper_o1_$$" "$TMPDIR/bench_viper_o1_$$_bin"
    fi

    # Viper AOT -O2
    viper_o2_time="N/A"
    if [ -f "$file" ]; then
        "$VIPER_BIN" build -O2 "$file" -o "$TMPDIR/bench_viper_o2_$$" 2>/dev/null
        viper_o2_time=$(run_bench "$TMPDIR/bench_viper_o2_$$_bin" "$ITERATIONS")
        rm -f "$TMPDIR/bench_viper_o2_$$" "$TMPDIR/bench_viper_o2_$$_bin"
    fi

    # Viper AOT -O3
    viper_o3_time="N/A"
    if [ -f "$file" ]; then
        "$VIPER_BIN" build -O3 "$file" -o "$TMPDIR/bench_viper_o3_$$" 2>/dev/null
        viper_o3_time=$(run_bench "$TMPDIR/bench_viper_o3_$$_bin" "$ITERATIONS")
        rm -f "$TMPDIR/bench_viper_o3_$$" "$TMPDIR/bench_viper_o3_$$_bin"
    fi

    # C -O3
    c_time="N/A"
    if [ -f "$c_file" ]; then
        gcc -O3 -march=native -flto -o "$TMPDIR/bench_c_$$" "$c_file" 2>/dev/null
        c_time=$(run_bench "$TMPDIR/bench_c_$$" "$ITERATIONS")
        rm -f "$TMPDIR/bench_c_$$"
    fi

    # Rust -O3
    rust_time="N/A"
    if [ -f "$rust_file" ]; then
        rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$TMPDIR/bench_rs_$$" "$rust_file" 2>/dev/null
        rust_time=$(run_bench "$TMPDIR/bench_rs_$$" "$ITERATIONS")
        rm -f "$TMPDIR/bench_rs_$$"
    fi

    # Go
    go_time="N/A"
    if [ -f "$go_file" ]; then
        go build -ldflags="-s -w" -o "$TMPDIR/bench_go_$$" "$go_file" 2>/dev/null
        go_time=$(run_bench "$TMPDIR/bench_go_$$" "$ITERATIONS")
        rm -f "$TMPDIR/bench_go_$$"
    fi

    # Print results table
    printf "\n  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "Language" "JIT" "O1" "O2" "O3" "C" "Rust" "Go"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "------------------" "--------" "--------" "--------" "--------" "--------" "--------" "--------"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "Time (ms)" "$viper_jit_time" "$viper_o1_time" "$viper_o2_time" "$viper_o3_time" "$c_time" "$rust_time" "$go_time"
    echo
}

# Run all benchmarks with full comparison (default mode)
run_all_full() {
    echo -e "${MAGENTA}========================================${NC}"
    echo -e "${MAGENTA}Cross-Language Benchmark Suite${NC}"
    echo -e "${MAGENTA}(JIT + AOT O1/O2/O3 + C/Rust/Go)${NC}"
    echo -e "${MAGENTA}Iterations: $ITERATIONS${NC}"
    echo -e "${MAGENTA}Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${MAGENTA}========================================${NC}"
    echo

    for bench in "${BENCHMARKS[@]}"; do
        if [[ -z "$1" || "$1" == "$bench" || "$1" == "all" ]]; then
            run_benchmark_full "$bench"
        fi
    done

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}All benchmarks complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

# Show help
show_help() {
    echo "Usage: $0 [OPTIONS] [BENCHMARK]"
    echo
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -i, --iterations N  Set number of iterations (default: 3)"
    echo
    echo "Default: Run all benchmarks with JIT, AOT (O1/O2/O3), C, Rust, Go comparison"
    echo
    echo "Benchmarks:"
    for bench in "${BENCHMARKS[@]}"; do
        echo "  $bench"
    done
    echo "  all                 Run all benchmarks (default)"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -i|--iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        *)
            BENCHMARK_TO_RUN="$1"
            shift
            ;;
    esac
done

check_prereqs
run_all_full "${BENCHMARK_TO_RUN:-all}"
