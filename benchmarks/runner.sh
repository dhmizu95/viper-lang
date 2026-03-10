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

    if ! command -v /usr/bin/time &> /dev/null; then
        echo -e "${YELLOW}Warning: /usr/bin/time not found, memory metrics disabled${NC}"
    fi

    if [ ! -f "$VIPER_BIN" ]; then
        echo -e "${YELLOW}Warning: Viper binary not found. Building...${NC}"
        cd "$PROJECT_ROOT" && cargo build --release
    fi

    echo -e "${GREEN}All prerequisites met${NC}"
    echo
}

# Time a benchmark run and measure memory (returns: time_ms,mem_kb)
run_bench_with_mem() {
    local cmd=$1
    local runs=$2
    local output_file="$TMPDIR/bench_out_$$"

    total_ms_int=0
    total_mem_kb=0
    valid_runs=0

    for i in $(seq 1 $runs); do
        if command -v /usr/bin/time &> /dev/null; then
            # Use GNU time for memory measurement
            /usr/bin/time -f "%e %M" -o "$output_file" sh -c "$cmd" > /dev/null 2>&1 || true
            if [ -f "$output_file" ] && [ -s "$output_file" ]; then
                read elapsed_sec mem_kb < "$output_file"
                # Skip if command crashed (mem_kb would be 0 or invalid)
                if [ -n "$elapsed_sec" ] && [ -n "$mem_kb" ] && [ "$mem_kb" -gt 0 ] 2>/dev/null; then
                    elapsed_ms_int=$(awk "BEGIN {printf \"%.0f\", $elapsed_sec * 1000}")
                    total_ms_int=$((total_ms_int + elapsed_ms_int))
                    total_mem_kb=$((total_mem_kb + mem_kb))
                    valid_runs=$((valid_runs + 1))
                fi
            fi
        else
            # Fallback without memory measurement
            start=$(date +%s%N)
            sh -c "$cmd" > /dev/null 2>&1 || true
            end=$(date +%s%N)
            elapsed=$((end - start))
            elapsed_ms_int=$((elapsed / 1000000))
            total_ms_int=$((total_ms_int + elapsed_ms_int))
            valid_runs=$((valid_runs + 1))
        fi
    done

    rm -f "$output_file"

    # Calculate average with one decimal place
    if [ $valid_runs -gt 0 ]; then
        avg_ms_int=$((total_ms_int / valid_runs))
        avg_ms_rem=$((total_ms_int % valid_runs))
        # Add decimal
        decimal=$((avg_ms_rem * 10 / valid_runs))
        avg_ms="${avg_ms_int}.${decimal}"
    else
        avg_ms="CRASH"
    fi

    if [ $total_mem_kb -gt 0 ]; then
        avg_mem_kb=$((total_mem_kb / valid_runs))
        echo "$avg_ms,$avg_mem_kb"
    else
        echo "$avg_ms,-"
    fi
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
    echo "  Timing & Memory (avg of $ITERATIONS runs):"

    # Viper JIT
    viper_jit_result="N/A,-"
    if [ -f "$file" ]; then
        viper_jit_result=$(run_bench_with_mem "$VIPER_BIN run -O3 $file" "$ITERATIONS")
    fi

    # Viper AOT -O1
    viper_o1_result="N/A,-"
    if [ -f "$file" ]; then
        "$VIPER_BIN" build -O1 "$file" -o "$TMPDIR/bench_viper_o1_$$" 2>/dev/null
        viper_o1_result=$(run_bench_with_mem "$TMPDIR/bench_viper_o1_$$_bin" "$ITERATIONS")
        rm -f "$TMPDIR/bench_viper_o1_$$" "$TMPDIR/bench_viper_o1_$$_bin"
    fi

    # Viper AOT -O2
    viper_o2_result="N/A,-"
    if [ -f "$file" ]; then
        "$VIPER_BIN" build -O2 "$file" -o "$TMPDIR/bench_viper_o2_$$" 2>/dev/null
        viper_o2_result=$(run_bench_with_mem "$TMPDIR/bench_viper_o2_$$_bin" "$ITERATIONS")
        rm -f "$TMPDIR/bench_viper_o2_$$" "$TMPDIR/bench_viper_o2_$$_bin"
    fi

    # Viper AOT -O3
    viper_o3_result="N/A,-"
    if [ -f "$file" ]; then
        "$VIPER_BIN" build -O3 "$file" -o "$TMPDIR/bench_viper_o3_$$" 2>/dev/null
        viper_o3_result=$(run_bench_with_mem "$TMPDIR/bench_viper_o3_$$_bin" "$ITERATIONS")
        rm -f "$TMPDIR/bench_viper_o3_$$" "$TMPDIR/bench_viper_o3_$$_bin"
    fi

    # C -O3
    c_result="N/A,-"
    if [ -f "$c_file" ]; then
        gcc -O3 -march=native -flto -o "$TMPDIR/bench_c_$$" "$c_file" 2>/dev/null
        c_result=$(run_bench_with_mem "$TMPDIR/bench_c_$$" "$ITERATIONS")
        rm -f "$TMPDIR/bench_c_$$"
    fi

    # Rust -O3
    rust_result="N/A,-"
    if [ -f "$rust_file" ]; then
        rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$TMPDIR/bench_rs_$$" "$rust_file" 2>/dev/null
        rust_result=$(run_bench_with_mem "$TMPDIR/bench_rs_$$" "$ITERATIONS")
        rm -f "$TMPDIR/bench_rs_$$"
    fi

    # Go
    go_result="N/A,-"
    if [ -f "$go_file" ]; then
        go build -ldflags="-s -w" -o "$TMPDIR/bench_go_$$" "$go_file" 2>/dev/null
        go_result=$(run_bench_with_mem "$TMPDIR/bench_go_$$" "$ITERATIONS")
        rm -f "$TMPDIR/bench_go_$$"
    fi

    # Parse results
    viper_jit_time=$(echo "$viper_jit_result" | cut -d',' -f1)
    viper_jit_mem=$(echo "$viper_jit_result" | cut -d',' -f2)
    viper_o1_time=$(echo "$viper_o1_result" | cut -d',' -f1)
    viper_o1_mem=$(echo "$viper_o1_result" | cut -d',' -f2)
    viper_o2_time=$(echo "$viper_o2_result" | cut -d',' -f1)
    viper_o2_mem=$(echo "$viper_o2_result" | cut -d',' -f2)
    viper_o3_time=$(echo "$viper_o3_result" | cut -d',' -f1)
    viper_o3_mem=$(echo "$viper_o3_result" | cut -d',' -f2)
    c_time=$(echo "$c_result" | cut -d',' -f1)
    c_mem=$(echo "$c_result" | cut -d',' -f2)
    rust_time=$(echo "$rust_result" | cut -d',' -f1)
    rust_mem=$(echo "$rust_result" | cut -d',' -f2)
    go_time=$(echo "$go_result" | cut -d',' -f1)
    go_mem=$(echo "$go_result" | cut -d',' -f2)

    # Print time table
    printf "\n  ${CYAN}Time (ms):${NC}\n"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "Language" "JIT" "O1" "O2" "O3" "C" "Rust" "Go"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "------------------" "--------" "--------" "--------" "--------" "--------" "--------" "--------"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "Time" "$viper_jit_time" "$viper_o1_time" "$viper_o2_time" "$viper_o3_time" "$c_time" "$rust_time" "$go_time"
    echo

    # Print memory table
    printf "  ${CYAN}Memory (KB):${NC}\n"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "Language" "JIT" "O1" "O2" "O3" "C" "Rust" "Go"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "------------------" "--------" "--------" "--------" "--------" "--------" "--------" "--------"
    printf "  %-18s %8s %8s %8s %8s %8s %8s %8s\n" \
        "Memory" "$viper_jit_mem" "$viper_o1_mem" "$viper_o2_mem" "$viper_o3_mem" "$c_mem" "$rust_mem" "$go_mem"
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
