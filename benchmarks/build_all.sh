#!/bin/bash
# Build script for all benchmarks in all four languages

set -e

BENCHMARK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCHMARK_DIR"

echo "========================================"
echo "Building all benchmarks"
echo "========================================"

# Build C benchmarks
echo ""
echo "=== Building C benchmarks ==="
for dir in */; do
    if [ -f "${dir}*.c" ]; then
        c_file=$(ls ${dir}*.c 2>/dev/null | head -1)
        if [ -n "$c_file" ]; then
            echo "Compiling $c_file..."
            gcc -O3 -o "${dir}benchmark_c" "$c_file" -lm 2>/dev/null || echo "  Skipped (compile error)"
        fi
    fi
done

# Build Go benchmarks
echo ""
echo "=== Building Go benchmarks ==="
for dir in */; do
    go_file=$(ls ${dir}*.go 2>/dev/null | head -1)
    if [ -n "$go_file" ]; then
        echo "Compiling $go_file..."
        (cd "$dir" && go build -o benchmark_go "$go_file" 2>/dev/null) || echo "  Skipped (compile error)"
    fi
done

# Build Rust benchmarks
echo ""
echo "=== Building Rust benchmarks ==="
for dir in */; do
    rs_file=$(ls ${dir}*.rs 2>/dev/null | head -1)
    if [ -n "$rs_file" ]; then
        echo "Compiling $rs_file..."
        rustc -O -o "${dir}benchmark_rs" "$rs_file" 2>/dev/null || echo "  Skipped (compile error)"
    fi
done

# Build Viper benchmarks (check if compiler exists)
echo ""
echo "=== Building Viper benchmarks ==="
if command -v viper &> /dev/null; then
    for dir in */; do
        vp_file=$(ls ${dir}*.vp 2>/dev/null | head -1)
        if [ -n "$vp_file" ]; then
            echo "Compiling $vp_file..."
            viper build "$vp_file" -o "${dir}benchmark_vp" 2>/dev/null || echo "  Skipped (compile error)"
        fi
    done
else
    echo "Viper compiler not found in PATH. Skipping Viper benchmarks."
    echo "Build viper first with: cargo build --release"
    echo "Then add to PATH or copy to /usr/local/bin"
fi

echo ""
echo "========================================"
echo "Build complete!"
echo "========================================"
