#!/bin/bash
# Build Viper benchmarks

BENCHMARK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCHMARK_DIR"

# Find viper compiler
VIPER_BIN=""
if command -v viper &> /dev/null; then
    VIPER_BIN="viper"
elif [ -f "$BENCHMARK_DIR/../target/release/viper" ]; then
    VIPER_BIN="$BENCHMARK_DIR/../target/release/viper"
elif [ -f "$BENCHMARK_DIR/../target/debug/viper" ]; then
    VIPER_BIN="$BENCHMARK_DIR/../target/debug/viper"
fi

if [ -z "$VIPER_BIN" ]; then
    echo "Error: Viper compiler not found!"
    echo "Please build viper first: cd .. && cargo build --release"
    exit 1
fi

echo "Using Viper compiler: $VIPER_BIN"
echo "Building Viper benchmarks..."

for dir in */; do
    vp_file=$(ls ${dir}*.vp 2>/dev/null | head -1)
    if [ -n "$vp_file" ]; then
        echo "  Compiling: $vp_file"
        "$VIPER_BIN" build "$vp_file" -o "${dir}benchmark_vp" 2>&1 || echo "    Failed (see errors above)"
    fi
done

echo "Done!"
