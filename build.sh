#!/bin/bash
# Viper Language Build Script
# Unified interface for building, testing, and benchmarking

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
DO_CLEAN=false
DO_BUILD=true
DO_TEST=false
DO_BENCHMARK=false
DO_INSTALL=false

# Helper functions
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

show_help() {
    cat << EOF
🐍 Viper Language Build Script
==============================

Usage: ./build.sh [OPTIONS]

Options:
  --clean       Clean build artifacts before building
  --test        Run tests (unit + integration)
  --benchmark   Run benchmarks
  --install     Install Viper globally
  --help        Show this help message

Examples:
  ./build.sh                    # Build only
  ./build.sh --clean            # Clean and build
  ./build.sh --test             # Build and run tests
  ./build.sh --clean --test     # Clean, build, and test
  ./build.sh --benchmark        # Build and run benchmarks
  ./build.sh --install          # Build and install globally
  ./build.sh --install --test   # Build, test, then install
  ./build.sh --clean --test --benchmark  # Full CI: clean, build, test, benchmark

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            DO_CLEAN=true
            shift
            ;;
        --test)
            DO_TEST=true
            shift
            ;;
        --benchmark)
            DO_BENCHMARK=true
            shift
            ;;
        --install)
            DO_INSTALL=true
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# If no action specified, default to build only
if [ "$DO_TEST" = false ] && [ "$DO_BENCHMARK" = false ] && [ "$DO_INSTALL" = false ]; then
    DO_BUILD=true
fi

echo ""
echo "🐍 Viper Build Script"
echo "====================="
echo ""

# ============================================
# Clean
# ============================================

if [ "$DO_CLEAN" = true ]; then
    print_info "Cleaning build artifacts..."
    if make clean; then
        print_success "Clean complete"
    else
        print_error "Clean failed"
        exit 1
    fi
    echo ""
fi

# ============================================
# Build (Runtime + Compiler)
# ============================================

# Build is required for test, benchmark, and install
DO_ACTUAL_BUILD=false
if [ "$DO_BUILD" = true ] || [ "$DO_TEST" = true ] || [ "$DO_BENCHMARK" = true ] || [ "$DO_INSTALL" = true ]; then
    DO_ACTUAL_BUILD=true
fi

if [ "$DO_ACTUAL_BUILD" = true ]; then
    # Build runtime library
    print_info "Building runtime library..."
    if [ ! -f "runtime/obj/libviper.a" ]; then
        print_info "Runtime library missing, forcing rebuild..."
        cd runtime && make clean && make
        cd ..
    else
        make runtime
    fi
    if [ -f "runtime/obj/libviper.a" ]; then
        print_success "Runtime build complete"
    else
        print_error "Runtime build failed - libviper.a not created"
        exit 1
    fi
    echo ""

    # Build compiler
    print_info "Building Viper compiler..."
    if make build; then
        print_success "Build complete"
    else
        print_error "Build failed"
        exit 1
    fi
    echo ""
fi

# ============================================
# Test
# ============================================

if [ "$DO_TEST" = true ]; then
    print_info "Running unit tests..."
    echo ""
    
    UNIT_RESULT=0
    cargo test --test unit 2>&1 | tee /tmp/unit_test.log
    if grep -q "test result: ok" /tmp/unit_test.log; then
        UNIT_RESULT=0
    else
        UNIT_RESULT=1
    fi
    
    # Extract unit test result
    UNIT_SUMMARY=$(grep "test result:" /tmp/unit_test.log | tail -1)
    
    echo ""
    print_info "Running integration tests..."
    echo ""
    
    INTEGRATION_RESULT=0
    cargo test --test integration 2>&1 | tee /tmp/integration_test.log
    if grep -q "test result: ok" /tmp/integration_test.log; then
        INTEGRATION_RESULT=0
    else
        INTEGRATION_RESULT=1
    fi
    
    # Extract integration test result
    INTEGRATION_SUMMARY=$(grep "test result:" /tmp/integration_test.log | tail -1)
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Test Results Summary"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Unit Tests:"
    echo "  $UNIT_SUMMARY"
    echo ""
    echo "Integration Tests:"
    echo "  $INTEGRATION_SUMMARY"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Clean up
    rm -f /tmp/unit_test.log /tmp/integration_test.log
    
    # Exit with failure if any tests failed
    if [ $UNIT_RESULT -ne 0 ] || [ $INTEGRATION_RESULT -ne 0 ]; then
        print_error "Some tests failed"
        exit 1
    else
        print_success "All tests passed"
    fi
    echo ""
fi

# ============================================
# Benchmark
# ============================================

if [ "$DO_BENCHMARK" = true ]; then
    print_info "Running benchmarks..."
    if make bench-safe; then
        print_success "Benchmarks complete"
    else
        print_error "Benchmarks failed"
        exit 1
    fi
    echo ""
fi

# ============================================
# Install
# ============================================

if [ "$DO_INSTALL" = true ]; then
    print_info "Installing Viper..."
    if [ -f "./install.sh" ]; then
        if ./install.sh; then
            print_success "Installation complete"
        else
            print_error "Installation failed"
            exit 1
        fi
    else
        print_error "install.sh not found"
        exit 1
    fi
    echo ""
fi

# ============================================
# Summary
# ============================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
print_success "Build script completed successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$DO_BUILD" = true ]; then
    echo "📦 Binary: target/debug/viper"
    echo ""
fi

echo "🚀 Quick Start:"
echo "   cargo run --bin viper -- run <file.vp>    # Run a Viper program"
echo "   cargo run --bin viper -- build <file.vp>  # Build an executable"
echo ""
