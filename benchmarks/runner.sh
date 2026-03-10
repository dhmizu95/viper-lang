#!/bin/bash
# Cross-Language Benchmark Runner (JIT mode)
# Compares Viper JIT, C, Rust, and Go performance

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ITERATIONS=${ITERATIONS:-3}
VIPER_BIN="$PROJECT_ROOT/target/release/viper"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Benchmark list
BENCHMARKS=(
    "01_fibonacci"
    "02_prime_sieve"
    "03_matrix_mul"
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
    local binary=$1
    local runs=$2
    
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

# Run single benchmark with timing
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
    gcc -O3 -march=native -flto -o "/tmp/bench_c_$$" "$SCRIPT_DIR/c/${name}.c" 2>/dev/null
    c_time=$(run_bench "/tmp/bench_c_$$" "$ITERATIONS")
    rm -f "/tmp/bench_c_$$"
    
    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "/tmp/bench_rs_$$" "$SCRIPT_DIR/rust/${name}.rs" 2>/dev/null
    rust_time=$(run_bench "/tmp/bench_rs_$$" "$ITERATIONS")
    rm -f "/tmp/bench_rs_$$"
    
    go build -ldflags="-s -w" -o "/tmp/bench_go_$$" "$SCRIPT_DIR/go/${name}.go" 2>/dev/null
    go_time=$(run_bench "/tmp/bench_go_$$" "$ITERATIONS")
    rm -f "/tmp/bench_go_$$"
    
    printf "    %-15s %8s ms\n" "Viper JIT:" "$viper_time"
    printf "    %-15s %8s ms\n" "C -O3:" "$c_time"
    printf "    %-15s %8s ms\n" "Rust -O3:" "$rust_time"
    printf "    %-15s %8s ms\n" "Go:" "$go_time"
    echo
}

# Run all benchmarks
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
run_all "${BENCHMARK_TO_RUN:-all}"
