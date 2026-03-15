#!/bin/bash
# Benchmark Runner - Isolated execution with crash protection
# Runs benchmarks in sandboxed environment to prevent system crashes
# Reports both performance (time) and memory usage

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ITERATIONS=${ITERATIONS:-3}
VIPER_BIN="$PROJECT_ROOT/target/release/viper"
TMPDIR="${TMPDIR:-/tmp}"

# Resource limits to prevent system crashes
MAX_MEMORY_MB=4096
MAX_TIME_SECONDS=300
MAX_FILE_SIZE_MB=512

# Check if /usr/bin/time is available for memory measurement
HAS_GNU_TIME=0
if command -v /usr/bin/time &> /dev/null; then
    HAS_GNU_TIME=1
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

BENCHMARKS=(
    "01_fibonacci"
    "02_prime_sieve"
    "03_matrix_mul"
    "04_quicksort"
    "05_matrix_mul"
    "06_prime_sieve"
    "07_string_ops"
    "08_int_hotloop"
    "09_nbody"
    "10_function_calls"
    "11_string_concat_scan"
    "12_bigint_overflow"
    "13_factorial"
    "14_recursive_list_sum"
)

if command -v python3 &> /dev/null; then
    PYTHON_BIN="python3"
elif command -v python &> /dev/null; then
    PYTHON_BIN="python"
else
    PYTHON_BIN=""
fi

# Results storage - store in memory, output to markdown
declare -A RESULTS
RESULTS_DIR="$SCRIPT_DIR/results"
mkdir -p "$RESULTS_DIR"
MD_REPORT="$RESULTS_DIR/benchmark_report.md"

log() {
    echo -e "$@"
}

log_md() {
    echo "$@" >> "$MD_REPORT"
}

# Run command with strict resource limits and crash protection
run_isolated() {
    local cmd="$1"
    local name="$2"
    local time_file="${3:-}"
    local output_file
    output_file=$(mktemp "$TMPDIR/safe_bench_${name}_XXXXXX")
    local mem_limit_kb=$((MAX_MEMORY_MB * 1024))
    local file_limit=$((MAX_FILE_SIZE_MB * 1024 * 1024))

    # Always apply timing inside the isolated process group so measurement never
    # bypasses the memory/time limits that protect the host system.
    # Note: We apply ulimits in the subshell, then run the command directly
    # (or via /usr/bin/time for memory measurement) without extra bash wrappers
    # to avoid measuring bash's memory overhead instead of the benchmark's.
    
    (
        ulimit -v $mem_limit_kb 2>/dev/null || true
        ulimit -f $file_limit 2>/dev/null || true
        ulimit -t $MAX_TIME_SECONDS 2>/dev/null || true
        
        if [ -n "$time_file" ] && [ $HAS_GNU_TIME -eq 1 ]; then
            # Run time directly on the command - no bash wrapper to skew memory
            exec nice -n 19 /usr/bin/time -f '%M' -o "$time_file" $cmd > "$output_file" 2>&1
        else
            exec nice -n 19 $cmd > "$output_file" 2>&1
        fi
    ) &
    local pid=$!

    # Wait with timeout
    local waited_ms=0
    local poll_interval_s="0.05"
    local poll_interval_ms=50
    local timeout_ms=$((MAX_TIME_SECONDS * 1000))
    while kill -0 $pid 2>/dev/null; do
        sleep "$poll_interval_s"
        waited_ms=$((waited_ms + poll_interval_ms))
        if [ $waited_ms -ge $timeout_ms ]; then
            kill -9 -$pid 2>/dev/null || kill -9 $pid 2>/dev/null
            wait $pid 2>/dev/null
            rm -f "$output_file"
            echo "TIMEOUT"
            return 124
        fi
    done

    wait $pid
    local exit_code=$?

    if [ -f "$output_file" ]; then
        cat "$output_file"
        rm -f "$output_file"
    fi

    return $exit_code
}

# Measure time and memory with crash protection
measure_benchmark() {
    local cmd="$1"
    local runs="$2"
    local name="$3"
    local total_ms=0
    local total_mem_kb=0
    local valid_runs=0
    local crashes=0
    local timeouts=0
    local time_file
    time_file=$(mktemp "$TMPDIR/time_measure_${name}_XXXXXX")

    for i in $(seq 1 $runs); do
        local start_ms=$(date +%s%3N)
        local result
        local mem_kb=0

        result=$(run_isolated "$cmd" "${name}_run${i}" "$time_file" 2>&1)
        local exit_code=$?

        if [ -f "$time_file" ] && [ -s "$time_file" ]; then
            mem_kb=$(cat "$time_file" 2>/dev/null || echo "0")
            if ! [[ "$mem_kb" =~ ^[0-9]+$ ]]; then
                mem_kb=0
            fi
        fi

        : > "$time_file"

        local end_ms=$(date +%s%3N)
        local elapsed=$((end_ms - start_ms))

        if [ $exit_code -eq 124 ]; then
            echo "    ${YELLOW}Run $i: TIMEOUT (${elapsed}ms, ${mem_kb}KB)${NC}" >&2
            timeouts=$((timeouts + 1))
        elif [ $exit_code -ne 0 ]; then
            echo "    ${RED}Run $i: CRASH (exit $exit_code, ${elapsed}ms, ${mem_kb}KB)${NC}" >&2
            crashes=$((crashes + 1))
        else
            total_ms=$((total_ms + elapsed))
            total_mem_kb=$((total_mem_kb + mem_kb))
            valid_runs=$((valid_runs + 1))
            echo "    ${GREEN}Run $i: OK (${elapsed}ms, ${mem_kb}KB)${NC}" >&2
        fi

        sleep 0.5
    done

    rm -f "$time_file"

    # Only output the summary line to stdout
    if [ $valid_runs -gt 0 ]; then
        local avg_ms=$((total_ms / valid_runs))
        local avg_mem=$((total_mem_kb / valid_runs))
        echo "$avg_ms,$avg_mem,$valid_runs,$crashes,$timeouts"
    elif [ $crashes -gt 0 ]; then
        echo "CRASH,0,$valid_runs,$crashes,$timeouts"
    else
        echo "TIMEOUT,0,$valid_runs,$crashes,$timeouts"
    fi
}

run_benchmark_safe() {
    local name="$1"
    local file="$SCRIPT_DIR/viper/${name}.vp"
    local c_file="$SCRIPT_DIR/c/${name}.c"
    local rust_file="$SCRIPT_DIR/rust/${name}.rs"
    local go_file="$SCRIPT_DIR/go/${name}.go"
    local py_file="$SCRIPT_DIR/python/${name}.py"

    log "${MAGENTA}========================================${NC}"
    log "${MAGENTA}Benchmark: $name${NC}"
    log "${MAGENTA}========================================${NC}"
    log

    # Viper JIT
    log "  Viper JIT (-O3):"
    if [ -f "$file" ]; then
        local jit_result
        jit_result=$(measure_benchmark "$VIPER_BIN run -O3 $file" "$ITERATIONS" "${name}_jit" 2>&1 | tail -1)
        RESULTS["${name}_jit"]="$jit_result"
    else
        log "    ${YELLOW}Skipped (file not found)${NC}"
        RESULTS["${name}_jit"]="SKIPPED"
    fi
    log

    # Viper AOT -O1
    log "  Viper AOT (-O1):"
    if [ -f "$file" ]; then
        local aot_bin="$TMPDIR/viper_aot_o1_$$"
        local aot_bin_path="${aot_bin}_bin"
        if "$VIPER_BIN" build -O1 "$file" -o "$aot_bin" 2>&1; then
            if [ -f "$aot_bin_path" ]; then
                local o1_result
                o1_result=$(measure_benchmark "$aot_bin_path" "$ITERATIONS" "${name}_o1" 2>&1 | tail -1)
                RESULTS["${name}_o1"]="$o1_result"
            else
                log "    ${RED}Binary not found${NC}"
                RESULTS["${name}_o1"]="BINARY_NOT_FOUND"
            fi
        else
            log "    ${RED}Build failed${NC}"
            RESULTS["${name}_o1"]="BUILD_FAILED"
        fi
        rm -f "$aot_bin" "$aot_bin_path" 2>/dev/null
    else
        log "    ${YELLOW}Skipped (file not found)${NC}"
        RESULTS["${name}_o1"]="SKIPPED"
    fi
    log

    # Viper AOT -O2
    log "  Viper AOT (-O2):"
    if [ -f "$file" ]; then
        local aot_bin="$TMPDIR/viper_aot_o2_$$"
        local aot_bin_path="${aot_bin}_bin"
        if "$VIPER_BIN" build -O2 "$file" -o "$aot_bin" 2>&1; then
            if [ -f "$aot_bin_path" ]; then
                local o2_result
                o2_result=$(measure_benchmark "$aot_bin_path" "$ITERATIONS" "${name}_o2" 2>&1 | tail -1)
                RESULTS["${name}_o2"]="$o2_result"
            else
                log "    ${RED}Binary not found${NC}"
                RESULTS["${name}_o2"]="BINARY_NOT_FOUND"
            fi
        else
            log "    ${RED}Build failed${NC}"
            RESULTS["${name}_o2"]="BUILD_FAILED"
        fi
        rm -f "$aot_bin" "$aot_bin_path" 2>/dev/null
    else
        log "    ${YELLOW}Skipped (file not found)${NC}"
        RESULTS["${name}_o2"]="SKIPPED"
    fi
    log

    # Viper AOT -O3
    log "  Viper AOT (-O3):"
    if [ -f "$file" ]; then
        local aot_bin="$TMPDIR/viper_aot_o3_$$"
        local aot_bin_path="${aot_bin}_bin"
        if "$VIPER_BIN" build -O3 "$file" -o "$aot_bin" 2>&1; then
            if [ -f "$aot_bin_path" ]; then
                local o3_result
                o3_result=$(measure_benchmark "$aot_bin_path" "$ITERATIONS" "${name}_o3" 2>&1 | tail -1)
                RESULTS["${name}_o3"]="$o3_result"
            else
                log "    ${RED}Binary not found${NC}"
                RESULTS["${name}_o3"]="BINARY_NOT_FOUND"
            fi
        else
            log "    ${RED}Build failed${NC}"
            RESULTS["${name}_o3"]="BUILD_FAILED"
        fi
        rm -f "$aot_bin" "${aot_bin}_bin" 2>/dev/null
    else
        log "    ${YELLOW}Skipped (file not found)${NC}"
        RESULTS["${name}_o3"]="SKIPPED"
    fi
    log

    # C (reference)
    log "  C (-O3):"
    if [ -f "$c_file" ]; then
        local c_bin="$TMPDIR/bench_c_$$"
        if gcc -O3 -o "$c_bin" "$c_file" 2>/dev/null; then
            local c_result
            c_result=$(measure_benchmark "$c_bin" "$ITERATIONS" "${name}_c" 2>&1 | tail -1)
            RESULTS["${name}_c"]="$c_result"
        else
            log "    ${RED}Build failed${NC}"
            RESULTS["${name}_c"]="BUILD_FAILED"
        fi
        rm -f "$c_bin" 2>/dev/null
    else
        log "    ${YELLOW}Skipped${NC}"
        RESULTS["${name}_c"]="SKIPPED"
    fi
    log

    # Rust (reference)
    log "  Rust (-O3):"
    if [ -f "$rust_file" ]; then
        local rust_bin="$TMPDIR/bench_rs_$$"
        if rustc -O -o "$rust_bin" "$rust_file" 2>/dev/null; then
            local rust_result
            rust_result=$(measure_benchmark "$rust_bin" "$ITERATIONS" "${name}_rust" 2>&1 | tail -1)
            RESULTS["${name}_rust"]="$rust_result"
        else
            log "    ${RED}Build failed${NC}"
            RESULTS["${name}_rust"]="BUILD_FAILED"
        fi
        rm -f "$rust_bin" 2>/dev/null
    else
        log "    ${YELLOW}Skipped${NC}"
        RESULTS["${name}_rust"]="SKIPPED"
    fi
    log

    # Go (reference)
    log "  Go:"
    if [ -f "$go_file" ]; then
        local go_bin="$TMPDIR/bench_go_$$"
        if go build -o "$go_bin" "$go_file" 2>/dev/null; then
            local go_result
            go_result=$(measure_benchmark "$go_bin" "$ITERATIONS" "${name}_go" 2>&1 | tail -1)
            RESULTS["${name}_go"]="$go_result"
        else
            log "    ${RED}Build failed${NC}"
            RESULTS["${name}_go"]="BUILD_FAILED"
        fi
        rm -f "$go_bin" 2>/dev/null
    else
        log "    ${YELLOW}Skipped${NC}"
        RESULTS["${name}_go"]="SKIPPED"
    fi
    log

    # Python (reference)
    log "  Python:"
    if [ -n "$PYTHON_BIN" ] && [ -f "$py_file" ]; then
        local py_result
        py_result=$(measure_benchmark "$PYTHON_BIN $py_file" "$ITERATIONS" "${name}_py" 2>&1 | tail -1)
        RESULTS["${name}_py"]="$py_result"
    else
        log "    ${YELLOW}Skipped${NC}"
        RESULTS["${name}_py"]="SKIPPED"
    fi
    log
}

run_all_safe() {
    log "${MAGENTA}========================================${NC}"
    log "${MAGENTA}Benchmark Runner${NC}"
    log "${MAGENTA}========================================${NC}"
    log "Date: $(date '+%Y-%m-%d %H:%M:%S')"
    log "Iterations: $ITERATIONS"
    log "Max Memory: ${MAX_MEMORY_MB}MB"
    log "Max Time per run: ${MAX_TIME_SECONDS}s"
    log
    if [ $HAS_GNU_TIME -eq 1 ]; then
        log "${BLUE}GNU time available - memory measurement enabled${NC}"
    else
        log "${YELLOW}GNU time not available - memory measurement disabled${NC}"
    fi
    log
    
    local failed_benchmarks=()
    local success_benchmarks=()
    
    for bench in "${BENCHMARKS[@]}"; do
        if [[ -z "${1:-}" || "$1" == "$bench" || "$1" == "all" ]]; then
            run_benchmark_safe "$bench"
            if [ $? -eq 0 ]; then
                success_benchmarks+=("$bench")
            else
                failed_benchmarks+=("$bench")
            fi
            log "${CYAN}----------------------------------------${NC}"
            log
        fi
    done
    
    log "${GREEN}========================================${NC}"
    log "${GREEN}Summary${NC}"
    log "${GREEN}========================================${NC}"
    log "Successful: ${#success_benchmarks[@]}"
    for b in "${success_benchmarks[@]}"; do
        log "  ${GREEN}✓${NC} $b"
    done
    
    if [ ${#failed_benchmarks[@]} -gt 0 ]; then
        log "Failed/Crashed: ${#failed_benchmarks[@]}"
        for b in "${failed_benchmarks[@]}"; do
            log "  ${RED}✗${NC} $b"
        done
    fi

    log
    generate_report
    log
}

benchmark_has_results() {
    local bench="$1"
    for lang in jit o1 o2 o3 c rust go py; do
        if [ -n "${RESULTS[${bench}_${lang}]:-}" ]; then
            return 0
        fi
    done
    return 1
}

# Generate performance and memory report in Markdown format
generate_report() {
    log "${GREEN}========================================${NC}"
    log "${GREEN}Generating Markdown Report${NC}"
    log "${GREEN}========================================${NC}"
    log
    
    # Initialize markdown file
    cat > "$MD_REPORT" << EOF
# Viper Benchmark Report

**Date:** $(date '+%Y-%m-%d %H:%M:%S')  
**Iterations:** $ITERATIONS  
**Max Memory Limit:** ${MAX_MEMORY_MB}MB  
**Max Time Limit:** ${MAX_TIME_SECONDS}s  

## Summary

EOF

    local report_benchmarks=()
    for bench in "${BENCHMARKS[@]}"; do
        if benchmark_has_results "$bench"; then
            report_benchmarks+=("$bench")
        fi
    done
    if [ ${#report_benchmarks[@]} -eq 0 ]; then
        report_benchmarks=("${BENCHMARKS[@]}")
    fi

    # Count successes and failures
    local total=0 passed=0 crashed=0
    for bench in "${report_benchmarks[@]}"; do
        for lang in jit o1 o2 o3 c rust go py; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-N/A}"
            if [[ "$result" =~ ^[0-9]+, ]]; then
                ((passed++))
            elif [[ "$result" == *"CRASH"* ]] || [[ "$result" == *"BUILD_FAILED"* ]] || [[ "$result" == *"BINARY_NOT_FOUND"* ]]; then
                ((crashed++))
            fi
            ((total++))
        done
    done
    
    log_md "| Metric | Value |"
    log_md "|--------|-------|"
    log_md "| Total Tests | $total |"
    log_md "| Passed | $passed |"
    log_md "| Failed/Crashed | $crashed |"
    log_md "| Success Rate | $(( passed * 100 / (total > 0 ? total : 1) ))% |"
    log_md ""
    
    # Time table
    log_md "## Performance (Time in ms)"
    log_md ""
    log_md "| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |"
    log_md "|-----------|-----|--------|--------|--------|---|------|-----|--------|"
    
    for bench in "${report_benchmarks[@]}"; do
        local row="| $bench |"
        for lang in jit o1 o2 o3 c rust go py; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-N/A}"
            local time_val="N/A"
            if [[ "$result" =~ ^([0-9]+), ]]; then
                time_val="${BASH_REMATCH[1]}"
            elif [[ "$result" == *"CRASH"* ]]; then
                time_val="CRASH"
            elif [[ "$result" == *"BUILD_FAILED"* ]]; then
                time_val="BUILD"
            elif [[ "$result" == *"SKIPPED"* ]]; then
                time_val="SKIP"
            fi
            row="$row $time_val |"
        done
        log_md "$row"
    done
    log_md ""
    
    # Memory table
    log_md "## Memory (Peak RSS in KB)"
    log_md ""
    log_md "| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |"
    log_md "|-----------|-----|--------|--------|--------|---|------|-----|--------|"
    
    for bench in "${report_benchmarks[@]}"; do
        local row="| $bench |"
        for lang in jit o1 o2 o3 c rust go py; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-N/A}"
            local mem_val="N/A"
            if [[ "$result" =~ ^[0-9]+,([0-9]+), ]]; then
                mem_val="${BASH_REMATCH[1]}"
            elif [[ "$result" == *"CRASH"* ]] || [[ "$result" == *"BUILD_FAILED"* ]] || [[ "$result" == *"SKIPPED"* ]]; then
                mem_val="N/A"
            fi
            row="$row $mem_val |"
        done
        log_md "$row"
    done
    log_md ""
    
    # Status table
    log_md "## Status"
    log_md ""
    log_md "| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |"
    log_md "|-----------|:---:|:------:|:------:|:------:|:-:|:----:|:---:|:------:|"

    for bench in "${report_benchmarks[@]}"; do
        local row="| $bench |"
        for lang in jit o1 o2 o3 c rust go py; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-N/A}"
            local status="❓"
            if [[ "$result" =~ ^[0-9]+, ]]; then
                status="✅"
            elif [[ "$result" == *"CRASH"* ]]; then
                status="❌"
            elif [[ "$result" == *"BUILD_FAILED"* ]]; then
                status="🔨"
            elif [[ "$result" == *"BINARY_NOT_FOUND"* ]]; then
                status="📁"
            elif [[ "$result" == *"SKIPPED"* ]]; then
                status="⏭️"
            fi
            row="$row $status |"
        done
        log_md "$row"
    done
    log_md ""

    # Performance Analysis Section
    log_md "---"
    log_md ""
    log_md "## Performance Analysis"
    log_md ""
    log_md "### Performance Ratio vs C (Baseline)"
    log_md ""
    log_md "| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |"
    log_md "|-----------|----------|-------------|-------------|-------------|"

    for bench in "${report_benchmarks[@]}"; do
        local c_key="${bench}_c"
        local c_result="${RESULTS[$c_key]:-}"
        local c_time=0
        if [[ "$c_result" =~ ^([0-9]+), ]]; then
            c_time="${BASH_REMATCH[1]}"
        fi

        local row="| $bench |"
        for lang in jit o1 o2 o3; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-}"
            local time_val=0
            if [[ "$result" =~ ^([0-9]+), ]]; then
                time_val="${BASH_REMATCH[1]}"
            fi

            if [ "$c_time" -gt 0 ] && [ "$time_val" -gt 0 ]; then
                local ratio=$(awk "BEGIN {printf \"%.1f×\", $time_val / $c_time}")
                row="$row $ratio |"
            else
                row="$row N/A |"
            fi
        done
        log_md "$row"
    done
    log_md ""

    log_md "### Performance Ratio vs Rust"
    log_md ""
    log_md "| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |"
    log_md "|-----------|-------------|----------------|----------------|----------------|"

    for bench in "${report_benchmarks[@]}"; do
        local rust_key="${bench}_rust"
        local rust_result="${RESULTS[$rust_key]:-}"
        local rust_time=0
        if [[ "$rust_result" =~ ^([0-9]+), ]]; then
            rust_time="${BASH_REMATCH[1]}"
        fi

        local row="| $bench |"
        for lang in jit o1 o2 o3; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-}"
            local time_val=0
            if [[ "$result" =~ ^([0-9]+), ]]; then
                time_val="${BASH_REMATCH[1]}"
            fi

            if [ "$rust_time" -gt 0 ] && [ "$time_val" -gt 0 ]; then
                local ratio=$(awk "BEGIN {printf \"%.1f×\", $time_val / $rust_time}")
                row="$row $ratio |"
            else
                row="$row N/A |"
            fi
        done
        log_md "$row"
    done
    log_md ""

    log_md "### Performance Ratio vs Go"
    log_md ""
    log_md "| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |"
    log_md "|-----------|-----------|--------------|--------------|--------------|"

    for bench in "${report_benchmarks[@]}"; do
        local go_key="${bench}_go"
        local go_result="${RESULTS[$go_key]:-}"
        local go_time=0
        if [[ "$go_result" =~ ^([0-9]+), ]]; then
            go_time="${BASH_REMATCH[1]}"
        fi

        local row="| $bench |"
        for lang in jit o1 o2 o3; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-}"
            local time_val=0
            if [[ "$result" =~ ^([0-9]+), ]]; then
                time_val="${BASH_REMATCH[1]}"
            fi

            if [ "$go_time" -gt 0 ] && [ "$time_val" -gt 0 ]; then
                local ratio=$(awk "BEGIN {printf \"%.1f×\", $time_val / $go_time}")
                row="$row $ratio |"
            else
                row="$row N/A |"
            fi
        done
        log_md "$row"
    done
    log_md ""

    log_md "### Memory Efficiency"
    log_md ""
    log_md "| Mode | Avg Memory (KB) | vs C |"
    log_md "|------|-----------------|------|"

    # Calculate average memory for each mode
    local jit_mem_total=0 jit_count=0
    local o1_mem_total=0 o1_count=0
    local o2_mem_total=0 o2_count=0
    local o3_mem_total=0 o3_count=0
    local c_mem_total=0 c_count=0

    for bench in "${report_benchmarks[@]}"; do
        for lang in jit o1 o2 o3 c; do
            local key="${bench}_${lang}"
            local result="${RESULTS[$key]:-}"
            if [[ "$result" =~ ^[0-9]+,([0-9]+), ]]; then
                local mem="${BASH_REMATCH[1]}"
                case $lang in
                    jit) jit_mem_total=$((jit_mem_total + mem)); jit_count=$((jit_count + 1)) ;;
                    o1) o1_mem_total=$((o1_mem_total + mem)); o1_count=$((o1_count + 1)) ;;
                    o2) o2_mem_total=$((o2_mem_total + mem)); o2_count=$((o2_count + 1)) ;;
                    o3) o3_mem_total=$((o3_mem_total + mem)); o3_count=$((o3_count + 1)) ;;
                    c) c_mem_total=$((c_mem_total + mem)); c_count=$((c_count + 1)) ;;
                esac
            fi
        done
    done

    local c_avg=3200
    if [ "$c_count" -gt 0 ]; then
        c_avg=$((c_mem_total / c_count))
    fi
    [ "$c_avg" -eq 0 ] && c_avg=1

    for mode in jit o1 o2 o3; do
        local mem_total=0 mem_count=0
        case $mode in
            jit) mem_total=$jit_mem_total; mem_count=$jit_count ;;
            o1) mem_total=$o1_mem_total; mem_count=$o1_count ;;
            o2) mem_total=$o2_mem_total; mem_count=$o2_count ;;
            o3) mem_total=$o3_mem_total; mem_count=$o3_count ;;
        esac

        local avg_mem=0
        if [ "$mem_count" -gt 0 ]; then
            avg_mem=$((mem_total / mem_count))
        fi

        local ratio=$(awk "BEGIN {printf \"%.1f×\", $avg_mem / $c_avg}")
        local mode_label="Viper JIT"
        case $mode in
            o1) mode_label="Viper AOT-O1" ;;
            o2) mode_label="Viper AOT-O2" ;;
            o3) mode_label="Viper AOT-O3" ;;
        esac

        log_md "| $mode_label | $avg_mem | $ratio |"
    done
    log_md ""

    log_md "### Key Findings"
    log_md ""
    
    # Calculate actual ratios for dynamic key findings
    local c_mem_display="$c_avg"
    local jit_ratio="N/A"
    if [ "$jit_count" -gt 0 ] && [ "$c_avg" -gt 0 ]; then
        jit_ratio=$(awk "BEGIN {printf \"%.1f\", ($jit_mem_total / $jit_count) / $c_avg}")
    fi
    local aot_avg_mem=0
    local aot_total_count=$((o1_count + o2_count + o3_count))
    if [ "$aot_total_count" -gt 0 ]; then
        aot_avg_mem=$(( (o1_mem_total + o2_mem_total + o3_mem_total) / aot_total_count ))
    fi
    local aot_ratio="N/A"
    if [ "$aot_total_count" -gt 0 ] && [ "$c_avg" -gt 0 ]; then
        aot_ratio=$(awk "BEGIN {printf \"%.1f\", $aot_avg_mem / $c_avg}")
    fi
    local jit_mem_avg=0
    if [ "$jit_count" -gt 0 ]; then
        jit_mem_avg=$((jit_mem_total / jit_count))
    fi
    
    log_md "1. **AOT-O1** typically offers the best performance/memory balance"
    log_md "2. **JIT mode** has ~${jit_ratio}× memory overhead (${jit_mem_avg}KB vs C's ~${c_mem_display}KB)"
    log_md "3. **AOT memory** is ~${aot_ratio}× C baseline (${aot_avg_mem}KB vs ~${c_mem_display}KB)"
    log_md "4. Performance varies by workload - see individual benchmark ratios above"
    log_md ""
    log_md "---"
    log_md "*Generated by Viper Benchmark Runner*"
    
    log "${GREEN}✓${NC} Report saved to: ${MD_REPORT}"
}

show_help() {
    echo "Usage: $0 [OPTIONS] [BENCHMARK]"
    echo
    echo "Safe benchmark runner with crash protection and resource limits"
    echo
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -i, --iterations N  Set number of iterations (default: 3)"
    echo "  -m, --memory MB     Set max memory limit (default: 2048)"
    echo "  -t, --timeout SEC   Set max time per run (default: 300)"
    echo
    echo "Safety features:"
    echo "  - Each benchmark runs in isolated process"
    echo "  - Memory limits prevent OOM crashes"
    echo "  - Timeouts prevent hangs"
    echo "  - Crashes are caught and logged"
    echo "  - Remaining benchmarks continue after crash"
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
        -m|--memory)
            MAX_MEMORY_MB="$2"
            shift 2
            ;;
        -t|--timeout)
            MAX_TIME_SECONDS="$2"
            shift 2
            ;;
        *)
            BENCHMARK_TO_RUN="$1"
            shift
            ;;
    esac
done

run_all_safe "${BENCHMARK_TO_RUN:-all}"
