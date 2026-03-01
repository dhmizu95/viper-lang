#!/bin/bash
# Build Go benchmarks

BENCHMARK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCHMARK_DIR"

echo "Building Go benchmarks..."

for dir in */; do
    go_file=$(ls ${dir}*.go 2>/dev/null | head -1)
    if [ -n "$go_file" ]; then
        echo "  Compiling: $go_file"
        (cd "$dir" && go build -o benchmark_go "$go_file")
        if [ $? -eq 0 ]; then
            echo "    Success: ${dir}benchmark_go"
        else
            echo "    Failed!"
        fi
    fi
done

echo "Done!"
