#!/bin/bash
# Run and benchmark all programs in all four languages

set -e

BENCHMARK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BENCHMARK_DIR"

RESULTS_FILE="$BENCHMARK_DIR/benchmark_results.txt"

echo "========================================"
echo "Running all benchmarks"
echo "Results will be saved to: $RESULTS_FILE"
echo "========================================"

# Initialize results file
echo "BENCHMARK RESULTS" > "$RESULTS_FILE"
echo "===============" >> "$RESULTS_FILE"
echo "Date: $(date)" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Function to run benchmark and capture time
run_benchmark() {
    local name=$1
    local executable=$2
    local language=$3
    
    if [ ! -f "$executable" ]; then
        echo "  $language: Executable not found"
        echo "$name | $language | NOT BUILT" >> "$RESULTS_FILE"
        return
    fi
    
    echo "  Running $language..."
    
    # Run 3 times and take average
    total_time=0
    runs=3
    
    for i in $(seq 1 $runs); do
        # Use /usr/bin/time for precise timing
        start_time=$(date +%s.%N)
        timeout 300 "$executable" > /dev/null 2>&1 || true
        end_time=$(date +%s.%N)
        elapsed=$(echo "$end_time - $start_time" | bc)
        total_time=$(echo "$total_time + $elapsed" | bc)
    done
    
    avg_time=$(echo "scale=4; $total_time / $runs" | bc)
    echo "  $language: ${avg_time}s (average of $runs runs)"
    echo "$name | $language | ${avg_time}s" >> "$RESULTS_FILE"
}

# Run each benchmark
for dir in */; do
    benchmark_name=$(basename "$dir")
    echo ""
    echo "=== $benchmark_name ==="
    echo "" >> "$RESULTS_FILE"
    echo "Benchmark: $benchmark_name" >> "$RESULTS_FILE"
    
    run_benchmark "$benchmark_name" "${dir}benchmark_c" "C"
    run_benchmark "$benchmark_name" "${dir}benchmark_go" "Go"
    run_benchmark "$benchmark_name" "${dir}benchmark_rs" "Rust"
    run_benchmark "$benchmark_name" "${dir}benchmark_vp" "Viper"
done

echo ""
echo "========================================"
echo "All benchmarks complete!"
echo "Results saved to: $RESULTS_FILE"
echo "========================================"

# Display summary
echo ""
echo "SUMMARY:"
echo "========"
cat "$RESULTS_FILE"
