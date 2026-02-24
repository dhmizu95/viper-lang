#!/bin/bash
#
# PGO (Profile-Guided Optimization) build script for Viper compiler
#
# This script performs a complete PGO build cycle:
# 1. Build instrumented binary
# 2. Run profiling workloads
# 3. Merge profile data
# 4. Build final optimized binary
#
# Usage: ./scripts/pgo.sh [instrument|run|merge|build|all]
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PGO_DATA_DIR="$PROJECT_DIR/target/pgo-data"
PROFRAW_DIR="$PGO_DATA_DIR/raw"
PROFDATA_FILE="$PGO_DATA_DIR/merged.profdata"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_llvm_tools() {
    if ! command -v llvm-profdata &> /dev/null; then
        log_error "llvm-profdata not found. Please ensure LLVM tools are installed."
        log_info "On Ubuntu/Debian: sudo apt install llvm-20"
        exit 1
    fi
}

setup_dirs() {
    mkdir -p "$PROFRAW_DIR"
    log_info "PGO data directory: $PGO_DATA_DIR"
}

# Step 1: Build instrumented binary
instrument() {
    log_info "Building PGO-instrumented binary..."
    cd "$PROJECT_DIR"
    
    # Clean previous PGO data
    rm -rf "$PGO_DATA_DIR"
    setup_dirs
    
    # Build with PGO instrumentation
    # LLVM_PROFILE_FILE sets where the .profraw files will be written
    LLVM_PROFILE_FILE="$PROFRAW_DIR/viper-%p-%m.profraw" \
        cargo build --profile pgo-instrument --bin viper
    
    log_info "Instrumented binary built: $PROJECT_DIR/target/pgo-instrument/viper"
    log_info "Run your workloads to generate profile data"
}

# Step 2: Run profiling workloads
run() {
    log_info "Running profiling workloads..."
    
    INSTRUMENTED_BIN="$PROJECT_DIR/target/pgo-instrument/viper"
    
    if [ ! -f "$INSTRUMENTED_BIN" ]; then
        log_error "Instrumented binary not found. Run 'instrument' first."
        exit 1
    fi
    
    # Create output directory for compiled programs
    mkdir -p "$PROJECT_DIR/target/pgo-output"
    
    # Run example workloads
    log_info "Compiling and running example programs..."
    
    # Compile examples with the instrumented compiler
    for example in "$PROJECT_DIR/examples/"*.viper; do
        if [ -f "$example" ]; then
            basename=$(basename "$example" .viper)
            log_info "Processing: $basename"
            
            # Run the instrumented compiler on the example
            "$INSTRUMENTED_BIN" "$example" -o "$PROJECT_DIR/target/pgo-output/$basename" 2>/dev/null || true
            
            # Execute the compiled program if it exists
            if [ -f "$PROJECT_DIR/target/pgo-output/$basename" ]; then
                "$PROJECT_DIR/target/pgo-output/$basename" 2>/dev/null || true
            fi
        fi
    done
    
    # Run any benchmark workloads
    if [ -d "$PROJECT_DIR/benchmark" ]; then
        log_info "Running benchmarks..."
        for bench in "$PROJECT_DIR/benchmark/"*.viper; do
            if [ -f "$bench" ]; then
                basename=$(basename "$bench" .viper)
                log_info "Benchmark: $basename"
                
                "$INSTRUMENTED_BIN" "$bench" -o "$PROJECT_DIR/target/pgo-output/$bench" 2>/dev/null || true
                
                if [ -f "$PROJECT_DIR/target/pgo-output/$bench" ]; then
                    "$PROJECT_DIR/target/pgo-output/$bench" 2>/dev/null || true
                fi
            fi
        done
    fi
    
    log_info "Profile data generated in: $PROFRAW_DIR"
    ls -la "$PROFRAW_DIR"/*.profraw 2>/dev/null || log_warn "No .profraw files found"
}

# Step 3: Merge profile data
merge() {
    log_info "Merging profile data..."
    check_llvm_tools
    
    if [ ! -d "$PROFRAW_DIR" ] || [ -z "$(ls -A "$PROFRAW_DIR"/*.profraw 2>/dev/null)" ]; then
        log_error "No .profraw files found. Run 'instrument' and 'run' first."
        exit 1
    fi
    
    # Merge all profraw files into a single profdata file
    llvm-profdata merge -sparse "$PROFRAW_DIR"/*.profraw -o "$PROFDATA_FILE"
    
    log_info "Profile data merged: $PROFDATA_FILE"
    
    # Display profile summary
    log_info "Profile summary:"
    llvm-profdata show "$PROFDATA_FILE" 2>/dev/null | head -20 || true
}

# Step 4: Build final optimized binary
build() {
    log_info "Building PGO-optimized binary..."
    check_llvm_tools
    
    if [ ! -f "$PROFDATA_FILE" ]; then
        log_error "Profile data not found. Run 'merge' first."
        exit 1
    fi
    
    cd "$PROJECT_DIR"
    
    # Set the PGO profile file for the build
    RUSTFLAGS="-Cprofile-use=$PROFDATA_FILE" \
        cargo build --profile pgo --bin viper
    
    log_info "PGO-optimized binary built: $PROJECT_DIR/target/pgo/viper"
    
    # Show binary size comparison
    log_info "Binary sizes:"
    ls -lh "$PROJECT_DIR/target/release/viper" 2>/dev/null || true
    ls -lh "$PROJECT_DIR/target/pgo/viper" 2>/dev/null || true
}

# Full PGO cycle
all() {
    log_info "Starting full PGO build cycle..."
    instrument
    run
    merge
    build
    log_info "PGO build complete!"
}

# Show usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  instrument  - Build instrumented binary"
    echo "  run         - Run workloads to generate profile data"
    echo "  merge       - Merge profile data into profdata file"
    echo "  build       - Build final PGO-optimized binary"
    echo "  all         - Run complete PGO cycle (default)"
    echo "  clean       - Remove all PGO data"
    echo ""
    echo "Examples:"
    echo "  $0 all              # Full PGO build cycle"
    echo "  $0 instrument       # Build instrumented binary only"
    echo "  $0 instrument && <run your workloads> && $0 merge && $0 build"
}

# Clean PGO data
clean() {
    log_info "Cleaning PGO data..."
    rm -rf "$PGO_DATA_DIR"
    rm -rf "$PROJECT_DIR/target/pgo-instrument"
    rm -rf "$PROJECT_DIR/target/pgo"
    rm -rf "$PROJECT_DIR/target/pgo-output"
    log_info "PGO data cleaned"
}

# Main
case "${1:-all}" in
    instrument)
        instrument
        ;;
    run)
        run
        ;;
    merge)
        merge
        ;;
    build)
        build
        ;;
    all)
        all
        ;;
    clean)
        clean
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        log_error "Unknown command: $1"
        usage
        exit 1
        ;;
esac
