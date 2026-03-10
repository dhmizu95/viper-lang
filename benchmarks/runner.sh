#!/bin/bash
# Cross-Language Benchmark Runner (JIT and AOT modes)
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
    "05_matrix_mul_array"
    "06_prime_sieve_array"
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

# Run Viper JIT benchmark
run_viper_jit() {
    local name=$1
    local file="$SCRIPT_DIR/viper/${name}.vp"
    
    if [ ! -f "$file" ]; then
        echo -e "${RED}Viper file not found: $file${NC}"
        return 1
    fi
    
    echo -n "  Viper JIT:     "
    "$VIPER_BIN" run -O3 "$file" 2>&1 | grep -v "^>" | grep -v "^🐍" | grep -v "^✅" | grep -v "^   " | tail -1
}

# Run Viper AOT benchmark
run_viper_aot() {
    local name=$1
    local file="$SCRIPT_DIR/viper/${name}.vp"
    local binary="$TMPDIR/bench_viper_aot_$$"
    local binary_final="${binary}_bin"

    if [ ! -f "$file" ]; then
        echo -e "${RED}Viper file not found: $file${NC}"
        return 1
    fi

    # AOT compile with -O2
    "$VIPER_BIN" build -O2 "$file" -o "$binary" 2>/dev/null

    echo -n "  Viper AOT-O2:  "
    "$binary_final" 2>&1 | tail -1

    rm -f "$binary" "$binary_final"
}

# Run C benchmark
run_c() {
    local name=$1
    local file="$SCRIPT_DIR/c/${name}.c"
    local binary="/tmp/bench_c_$$"
    
    if [ ! -f "$file" ]; then
        echo -e "${RED}C file not found: $file${NC}"
        return 1
    fi
    
    gcc -O3 -march=native -flto -o "$binary" "$file" 2>/dev/null
    
    echo -n "  C (GCC -O3):   "
    "$binary" 2>&1 | tail -1
    
    rm -f "$binary"
}

# Run Rust benchmark
run_rust() {
    local name=$1
    local file="$SCRIPT_DIR/rust/${name}.rs"
    local binary="/tmp/bench_rs_$$"
    
    if [ ! -f "$file" ]; then
        echo -e "${RED}Rust file not found: $file${NC}"
        return 1
    fi
    
    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$binary" "$file" 2>/dev/null
    
    echo -n "  Rust (O3):     "
    "$binary" 2>&1 | tail -1
    
    rm -f "$binary"
}

# Run Go benchmark
run_go() {
    local name=$1
    local file="$SCRIPT_DIR/go/${name}.go"
    local binary="/tmp/bench_go_$$"
    
    if [ ! -f "$file" ]; then
        echo -e "${RED}Go file not found: $file${NC}"
        return 1
    fi
    
    go build -ldflags="-s -w" -o "$binary" "$file" 2>/dev/null
    
    echo -n "  Go:            "
    "$binary" 2>&1 | tail -1
    
    rm -f "$binary"
}

# Run single benchmark with timing (JIT mode)
run_benchmark() {
    local name=$1

    echo -e "${YELLOW}========================================${NC}"
    echo -e "${YELLOW}Benchmark: $name${NC}"
    echo -e "${YELLOW}========================================${NC}"
    echo

    # Run and show output
    run_viper_jit "$name" || true
    run_c "$name" || true
    run_rust "$name" || true
    run_go "$name" || true

    echo
    echo "  Timing (avg of $ITERATIONS runs):"

    # Time each
    viper_time=$(run_bench "$VIPER_BIN run -O3 $SCRIPT_DIR/viper/${name}.vp" "$ITERATIONS" 2>/dev/null || echo "N/A")

    # Compile and time others
    gcc -O3 -march=native -flto -o "$TMPDIR/bench_c_$$" "$SCRIPT_DIR/c/${name}.c" 2>/dev/null
    c_time=$(run_bench "$TMPDIR/bench_c_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_c_$$"

    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$TMPDIR/bench_rs_$$" "$SCRIPT_DIR/rust/${name}.rs" 2>/dev/null
    rust_time=$(run_bench "$TMPDIR/bench_rs_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_rs_$$"

    go build -ldflags="-s -w" -o "$TMPDIR/bench_go_$$" "$SCRIPT_DIR/go/${name}.go" 2>/dev/null
    go_time=$(run_bench "$TMPDIR/bench_go_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_go_$$"

    printf "    %-15s %8s ms\n" "Viper JIT:" "$viper_time"
    printf "    %-15s %8s ms\n" "C -O3:" "$c_time"
    printf "    %-15s %8s ms\n" "Rust -O3:" "$rust_time"
    printf "    %-15s %8s ms\n" "Go:" "$go_time"
    echo
}

# Run single benchmark with timing (AOT mode)
run_benchmark_aot() {
    local name=$1

    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}Benchmark: $name (AOT Mode)${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo

    # Run and show output
    run_viper_aot "$name" || true
    run_c "$name" || true
    run_rust "$name" || true
    run_go "$name" || true

    echo
    echo "  Timing (avg of $ITERATIONS runs):"

    # AOT compile and time Viper
    "$VIPER_BIN" build -O2 "$SCRIPT_DIR/viper/${name}.vp" -o "$TMPDIR/bench_viper_aot_$$" 2>/dev/null
    viper_aot_time=$(run_bench "$TMPDIR/bench_viper_aot_$$_bin" "$ITERATIONS")
    rm -f "$TMPDIR/bench_viper_aot_$$" "$TMPDIR/bench_viper_aot_$$_bin"

    # Compile and time others
    gcc -O3 -march=native -flto -o "$TMPDIR/bench_c_$$" "$SCRIPT_DIR/c/${name}.c" 2>/dev/null
    c_time=$(run_bench "$TMPDIR/bench_c_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_c_$$"

    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$TMPDIR/bench_rs_$$" "$SCRIPT_DIR/rust/${name}.rs" 2>/dev/null
    rust_time=$(run_bench "$TMPDIR/bench_rs_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_rs_$$"

    go build -ldflags="-s -w" -o "$TMPDIR/bench_go_$$" "$SCRIPT_DIR/go/${name}.go" 2>/dev/null
    go_time=$(run_bench "$TMPDIR/bench_go_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_go_$$"

    printf "    %-15s %8s ms\n" "Viper AOT-O2:" "$viper_aot_time"
    printf "    %-15s %8s ms\n" "C -O3:" "$c_time"
    printf "    %-15s %8s ms\n" "Rust -O3:" "$rust_time"
    printf "    %-15s %8s ms\n" "Go:" "$go_time"
    echo
}

# Run all benchmarks (JIT mode)
run_all() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Cross-Language Benchmark Suite (JIT)${NC}"
    echo -e "${BLUE}Iterations: $ITERATIONS${NC}"
    echo -e "${BLUE}Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo

    for bench in "${BENCHMARKS[@]}"; do
        if [[ -z "$1" || "$1" == "$bench" || "$1" == "all" ]]; then
            run_benchmark "$bench"
        fi
    done

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}All JIT benchmarks complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

# Run all benchmarks (AOT mode)
run_all_aot() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}Cross-Language Benchmark Suite (AOT)${NC}"
    echo -e "${CYAN}Iterations: $ITERATIONS${NC}"
    echo -e "${CYAN}Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo

    for bench in "${BENCHMARKS[@]}"; do
        if [[ -z "$1" || "$1" == "$bench" || "$1" == "all" ]]; then
            run_benchmark_aot "$bench"
        fi
    done

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}All AOT benchmarks complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

# Run both JIT and AOT benchmarks
run_all_both() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Cross-Language Benchmark Suite (JIT + AOT)${NC}"
    echo -e "${BLUE}Iterations: $ITERATIONS${NC}"
    echo -e "${BLUE}Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo

    for bench in "${BENCHMARKS[@]}"; do
        if [[ -z "$1" || "$1" == "$bench" || "$1" == "all" ]]; then
            run_benchmark "$bench"
            run_benchmark_aot "$bench"
        fi
    done

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}All benchmarks complete (JIT + AOT)!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

# Run comprehensive benchmark with all optimization levels
run_all_opt_levels() {
    local name=$1

    echo -e "${MAGENTA}========================================${NC}"
    echo -e "${MAGENTA}Benchmark: $name (All Opt Levels)${NC}"
    echo -e "${MAGENTA}========================================${NC}"
    echo

    # Run and show output (just once for verification)
    echo "  Output verification:"
    run_c "$name" || true

    echo
    echo "  Timing (avg of $ITERATIONS runs):"

    # Viper JIT
    viper_jit_time=$(run_bench "$VIPER_BIN run -O3 $SCRIPT_DIR/viper/${name}.vp" "$ITERATIONS" 2>/dev/null || echo "N/A")

    # Viper AOT -O1
    "$VIPER_BIN" build -O1 "$SCRIPT_DIR/viper/${name}.vp" -o "$TMPDIR/bench_viper_o1_$$" 2>/dev/null
    viper_o1_time=$(run_bench "$TMPDIR/bench_viper_o1_$$_bin" "$ITERATIONS")
    rm -f "$TMPDIR/bench_viper_o1_$$" "$TMPDIR/bench_viper_o1_$$_bin"

    # Viper AOT -O2
    "$VIPER_BIN" build -O2 "$SCRIPT_DIR/viper/${name}.vp" -o "$TMPDIR/bench_viper_o2_$$" 2>/dev/null
    viper_o2_time=$(run_bench "$TMPDIR/bench_viper_o2_$$_bin" "$ITERATIONS")
    rm -f "$TMPDIR/bench_viper_o2_$$" "$TMPDIR/bench_viper_o2_$$_bin"

    # Viper AOT -O3
    "$VIPER_BIN" build -O3 "$SCRIPT_DIR/viper/${name}.vp" -o "$TMPDIR/bench_viper_o3_$$" 2>/dev/null
    viper_o3_time=$(run_bench "$TMPDIR/bench_viper_o3_$$_bin" "$ITERATIONS")
    rm -f "$TMPDIR/bench_viper_o3_$$" "$TMPDIR/bench_viper_o3_$$_bin"

    # C -O3
    gcc -O3 -march=native -flto -o "$TMPDIR/bench_c_$$" "$SCRIPT_DIR/c/${name}.c" 2>/dev/null
    c_time=$(run_bench "$TMPDIR/bench_c_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_c_$$"

    # Rust -O3
    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$TMPDIR/bench_rs_$$" "$SCRIPT_DIR/rust/${name}.rs" 2>/dev/null
    rust_time=$(run_bench "$TMPDIR/bench_rs_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_rs_$$"

    # Go
    go build -ldflags="-s -w" -o "$TMPDIR/bench_go_$$" "$SCRIPT_DIR/go/${name}.go" 2>/dev/null
    go_time=$(run_bench "$TMPDIR/bench_go_$$" "$ITERATIONS")
    rm -f "$TMPDIR/bench_go_$$"

    # Print results table
    printf "\n  %-20s %10s\n" "Language" "Avg Time (ms)"
    printf "  %-20s %10s\n" "--------------------" "----------"
    printf "  %-20s %10s\n" "Viper JIT" "$viper_jit_time"
    printf "  %-20s %10s\n" "Viper AOT -O1" "$viper_o1_time"
    printf "  %-20s %10s\n" "Viper AOT -O2" "$viper_o2_time"
    printf "  %-20s %10s\n" "Viper AOT -O3" "$viper_o3_time"
    printf "  %-20s %10s\n" "C -O3" "$c_time"
    printf "  %-20s %10s\n" "Rust -O3" "$rust_time"
    printf "  %-20s %10s\n" "Go" "$go_time"
    echo
}

# Run all benchmarks with all optimization levels comparison
run_all_opt_comparison() {
    echo -e "${MAGENTA}========================================${NC}"
    echo -e "${MAGENTA}Cross-Language Benchmark Suite${NC}"
    echo -e "${MAGENTA}(All Optimization Levels)${NC}"
    echo -e "${MAGENTA}Iterations: $ITERATIONS${NC}"
    echo -e "${MAGENTA}Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${MAGENTA}========================================${NC}"
    echo

    for bench in "${BENCHMARKS[@]}"; do
        if [[ -z "$1" || "$1" == "$bench" || "$1" == "all" ]]; then
            run_all_opt_levels "$bench"
        fi
    done

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}All optimization level benchmarks complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

# Show help
show_help() {
    echo "Usage: $0 [OPTIONS] [BENCHMARK]"
    echo
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -i, --iterations N  Set number of iterations (default: 3)"
    echo "  --jit               Run JIT benchmarks only (default)"
    echo "  --aot               Run AOT benchmarks only (-O2)"
    echo "  --both              Run both JIT and AOT benchmarks"
    echo "  --opt-compare       Run all optimization levels (JIT, O1, O2, O3) + C/Rust/Go"
    echo
    echo "Benchmarks:"
    for bench in "${BENCHMARKS[@]}"; do
        echo "  $bench"
    done
    echo "  all                 Run all benchmarks (default)"
}

# Parse arguments
MODE="jit"
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
        --jit)
            MODE="jit"
            shift
            ;;
        --aot)
            MODE="aot"
            shift
            ;;
        --both)
            MODE="both"
            shift
            ;;
        --opt-compare)
            MODE="opt-compare"
            shift
            ;;
        *)
            BENCHMARK_TO_RUN="$1"
            shift
            ;;
    esac
done

check_prereqs

case $MODE in
    jit)
        run_all "${BENCHMARK_TO_RUN:-all}"
        ;;
    aot)
        run_all_aot "${BENCHMARK_TO_RUN:-all}"
        ;;
    both)
        run_all_both "${BENCHMARK_TO_RUN:-all}"
        ;;
    opt-compare)
        run_all_opt_comparison "${BENCHMARK_TO_RUN:-all}"
        ;;
esac
