#!/bin/bash
# Build C benchmarks

BENCHMARK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCHMARK_DIR"

echo "Building C benchmarks..."

for dir in */; do
    c_file=$(ls ${dir}*.c 2>/dev/null | head -1)
    if [ -n "$c_file" ]; then
        echo "  Compiling: $c_file"
        gcc -O3 -march=native -o "${dir}benchmark_c" "$c_file" -lm
        if [ $? -eq 0 ]; then
            echo "    Success: ${dir}benchmark_c"
        else
            echo "    Failed!"
        fi
    fi
done

echo "Done!"
