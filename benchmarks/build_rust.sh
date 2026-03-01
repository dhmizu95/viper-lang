#!/bin/bash
# Build Rust benchmarks

BENCHMARK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCHMARK_DIR"

echo "Building Rust benchmarks..."

for dir in */; do
    rs_file=$(ls ${dir}*.rs 2>/dev/null | head -1)
    if [ -n "$rs_file" ]; then
        echo "  Compiling: $rs_file"
        rustc -C opt-level=3 -C target-cpu=native -o "${dir}benchmark_rs" "$rs_file"
        if [ $? -eq 0 ]; then
            echo "    Success: ${dir}benchmark_rs"
        else
            echo "    Failed!"
        fi
    fi
done

echo "Done!"
