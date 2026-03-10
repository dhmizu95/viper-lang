#!/bin/bash
# Cross-Language Benchmark Runner
# Compares Viper, C, Rust, and Go performance

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$SCRIPT_DIR/results"
ITERATIONS=${ITERATIONS:-3}
VIPER_BIN="$PROJECT_ROOT/target/release/viper"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Benchmark list
BENCHMARKS=(
    "01_fibonacci"
    "02_prime_sieve"
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

# Run Viper benchmark
run_viper() {
    local name=$1
    local file="$SCRIPT_DIR/viper/${name}.vp"
    
    if [ ! -f "$file" ]; then
        echo -e "${RED}Viper file not found: $file${NC}"
        return 1
    fi
    
    echo -n "  Viper:         "
    local start=$(date +%s%N)
    "$VIPER_BIN" run -O3 "$file" 2>&1 | grep -v "^>" | grep -v "^🐍" | grep -v "^✅" | grep -v "^   " | tail -1
    local end=$(date +%s%N)
    local elapsed=$(( (end - start) / 1000000 ))
    echo "                 (${elapsed}ms total)"
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
    
    # Compile with optimizations
    gcc -O3 -march=native -flto -o "$binary" "$file" 2>/dev/null
    
    echo -n "  C (GCC -O3):   "
    local start=$(date +%s%N)
    "$binary" 2>&1 | tail -1
    local end=$(date +%s%N)
    local elapsed=$(( (end - start) / 1000000 ))
    echo "                 (${elapsed}ms total)"
    
    # Cleanup
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
    
    # Compile with optimizations
    rustc -C opt-level=3 -C lto=fat -C target-cpu=native -o "$binary" "$file" 2>/dev/null
    
    echo -n "  Rust (O3):     "
    local start=$(date +%s%N)
    "$binary" 2>&1 | tail -1
    local end=$(date +%s%N)
    local elapsed=$(( (end - start) / 1000000 ))
    echo "                 (${elapsed}ms total)"
    
    # Cleanup
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
    
    # Compile with optimizations
    go build -ldflags="-s -w" -o "$binary" "$file" 2>/dev/null
    
    echo -n "  Go:            "
    local start=$(date +%s%N)
    "$binary" 2>&1 | tail -1
    local end=$(date +%s%N)
    local elapsed=$(( (end - start) / 1000000 ))
    echo "                 (${elapsed}ms total)"
    
    # Cleanup
    rm -f "$binary"
}

# Run single benchmark
run_benchmark() {
    local name=$1
    
    echo -e "${YELLOW}========================================${NC}"
    echo -e "${YELLOW}Benchmark: $name${NC}"
    echo -e "${YELLOW}========================================${NC}"
    echo
    
    run_viper "$name" || true
    run_c "$name" || true
    run_rust "$name" || true
    run_go "$name" || true
    
    echo
}

# Run all benchmarks
run_all() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Cross-Language Benchmark Suite${NC}"
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
    echo
    echo "Examples:"
    echo "  $0                  Run all benchmarks"
    echo "  $0 01_fibonacci     Run Fibonacci benchmark only"
    echo "  $0 -i 10 all        Run all benchmarks with 10 iterations"
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

# Main
check_prereqs
run_all "${BENCHMARK_TO_RUN:-all}"
