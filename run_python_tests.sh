#!/bin/bash

VIPER=./target/debug/viper
TEST_DIR=tests/02_python_compat
OUTPUT_DIR=$TEST_DIR/results
SUMMARY_FILE=$OUTPUT_DIR/summary.txt
DETAILED_FILE=$OUTPUT_DIR/detailed.json

mkdir -p "$OUTPUT_DIR"

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_ERRORS=0
TOTAL_SKIP=0

declare -A CATEGORY_PASS
declare -A CATEGORY_FAIL
declare -A CATEGORY_TOTAL

init_category() {
    CATEGORY_PASS[$1]=0
    CATEGORY_FAIL[$1]=0
    CATEGORY_TOTAL[$1]=0
}

update_category() {
    local cat=$1
    local status=$2
    if [ -z "${CATEGORY_TOTAL[$cat]}" ]; then
        init_category "$cat"
    fi
    CATEGORY_TOTAL[$cat]=$((CATEGORY_TOTAL[$cat] + 1))
    if [ "$status" = "PASS" ]; then
        CATEGORY_PASS[$cat]=$((CATEGORY_PASS[$cat] + 1))
    else
        CATEGORY_FAIL[$cat]=$((CATEGORY_FAIL[$cat] + 1))
    fi
}

run_test() {
    local file="$1"
    local name=$(basename "$file" .vp)
    local category=$(basename "$(dirname "$file")")
    local expected="$2"
    
    CATEGORY_TOTAL[$category]=0
    CATEGORY_PASS[$category]=0
    CATEGORY_FAIL[$category]=0
    
    if [ -z "$expected" ]; then
        expected=""
    fi

    local bin="${name}_vp_bin"
    local start_time=$(date +%s%N)
    
    if ! $VIPER build "$file" >/dev/null 2>&1; then
        echo "❌ $category/$name: COMPILE ERROR"
        echo "{\"name\":\"$name\",\"category\":\"$category\",\"status\":\"COMPILE_ERROR\"}" >> "$DETAILED_FILE.tmp"
        TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
        update_category "$category" "FAIL"
        return
    fi

    if [ ! -f "$bin" ]; then
        echo "❌ $category/$name: BINARY NOT FOUND"
        echo "{\"name\":\"$name\",\"category\":\"$category\",\"status\":\"BINARY_NOT_FOUND\"}" >> "$DETAILED_FILE.tmp"
        TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
        update_category "$category" "FAIL"
        return
    fi

    local actual
    actual=$(timeout 5 ./"$bin" 2>&1) 
    local rc=$?
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    if [ $rc -eq 124 ]; then
        echo "⏱️  $category/$name: TIMEOUT"
        echo "{\"name\":\"$name\",\"category\":\"$category\",\"status\":\"TIMEOUT\",\"duration_ms\":$duration}" >> "$DETAILED_FILE.tmp"
        TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
        update_category "$category" "FAIL"
        return
    elif [ $rc -ne 0 ]; then
        echo "❌ $category/$name: RUNTIME ERROR (exit $rc)"
        echo "{\"name\":\"$name\",\"category\":\"$category\",\"status\":\"RUNTIME_ERROR\",\"exit_code\":$rc,\"output\":$(echo "$actual" | head -3 | tr -d '\n' | tr -s ' ')}" >> "$DETAILED_FILE.tmp"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
        update_category "$category" "FAIL"
        return
    fi

    if [ "$actual" = "$expected" ] || [ -z "$expected" ]; then
        echo "✅ $category/$name: PASS"
        echo "{\"name\":\"$name\",\"category\":\"$category\",\"status\":\"PASS\",\"duration_ms\":$duration}" >> "$DETAILED_FILE.tmp"
        TOTAL_PASS=$((TOTAL_PASS + 1))
        update_category "$category" "PASS"
    else
        echo "❌ $category/$name: WRONG OUTPUT"
        echo "   expected: $(echo "$expected" | head -1)"
        echo "   actual:   $(echo "$actual" | head -1)"
        echo "{\"name\":\"$name\",\"category\":\"$category\",\"status\":\"WRONG_OUTPUT\",\"expected\":$(echo "$expected" | head -1 | tr -d '\n' | tr -s ' ' | xxd -p),\"actual\":$(echo "$actual" | head -1 | tr -d '\n' | tr -s ' ' | xxd -p)}" >> "$DETAILED_FILE.tmp"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
        update_category "$category" "FAIL"
    fi
}

echo "=== Python Feature Test Suite ===" 
echo "Running tests..."
echo ""

: > "$DETAILED_FILE.tmp"

cargo build -q 2>/dev/null

for dir in $TEST_DIR/*/; do
    category=$(basename "$dir")
    if [ "$category" = "results" ]; then
        continue
    fi
    init_category "$category"
done

for file in $(find $TEST_DIR -name "*.vp" | sort); do
    run_test "$file" ""
done

echo ""
echo "=== Results: $TOTAL_PASS passed, $TOTAL_FAIL failed, $TOTAL_ERRORS errors ==="

{
    echo "=== Python Feature Test Summary ==="
    echo ""
    printf "%-24s | %-7s | %-7s | %-6s\n" "Category" "Passed" "Failed" "Total"
    echo "------------------------|----------|----------|--------"
    
    for cat in $(find $TEST_DIR -maxdepth 1 -type d -not -name "02_python_compat" -not -name "results" | xargs -n1 basename | sort); do
        pass=${CATEGORY_PASS[$cat]:-0}
        fail=${CATEGORY_FAIL[$cat]:-0}
        total=${CATEGORY_TOTAL[$cat]:-0}
        printf "%-24s | %7d | %7d | %6d\n" "$cat" "$pass" "$fail" "$total"
    done
    
    echo "------------------------|----------|----------|--------"
    printf "%-24s | %7d | %7d | %6d\n" "TOTAL" "$TOTAL_PASS" "$TOTAL_FAIL" $((TOTAL_PASS + TOTAL_FAIL + TOTAL_ERRORS))
} > "$SUMMARY_FILE"

cat "$SUMMARY_FILE"

echo ""
echo "Detailed results: $DETAILED_FILE"

{
    echo "{"
    echo "  \"timestamp\": \"$(date -Iseconds)\","
    echo "  \"total\": $((TOTAL_PASS + TOTAL_FAIL + TOTAL_ERRORS)),"
    echo "  \"passed\": $TOTAL_PASS,"
    echo "  \"failed\": $TOTAL_FAIL,"
    echo "  \"errors\": $TOTAL_ERRORS,"
    echo "  \"tests\": ["
    
    sed -e 's/^/    /' -e 's/}$/},/' "$DETAILED_FILE.tmp" | head -n -1
    
    echo "    {}"
    echo "  ]"
    echo "}"
} > "$DETAILED_FILE"

rm "$DETAILED_FILE.tmp"

echo ""
echo "Reports generated successfully!"
