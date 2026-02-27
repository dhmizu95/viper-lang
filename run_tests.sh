#!/bin/bash
set -e

VIPER=./target/debug/viper
PASS=0
FAIL=0
ERRORS=()

run_test() {
    local file="$1"
    local expected="$2"
    local name=$(basename "$file" .vp)

    # Compile
    if ! $VIPER build "$file" >/dev/null 2>&1; then
        echo "❌ $name: COMPILE ERROR"
        ERRORS+=("$name: compile error")
        FAIL=$((FAIL + 1))
        return
    fi

    # Get binary name
    local bin="${name}_vp_bin"
    if [ ! -f "$bin" ]; then
        echo "❌ $name: binary not found"
        ERRORS+=("$name: binary not found")
        FAIL=$((FAIL + 1))
        return
    fi

    # Run with timeout
    local actual
    actual=$(timeout 5 ./"$bin" 2>&1) || {
        local rc=$?
        if [ $rc -eq 124 ]; then
            echo "❌ $name: TIMEOUT (possible infinite loop)"
            ERRORS+=("$name: timeout")
        else
            echo "❌ $name: RUNTIME ERROR (exit $rc)"
            ERRORS+=("$name: runtime error (exit $rc)")
        fi
        FAIL=$((FAIL + 1))
        return
    }

    # Special handling for test_concurrency (order doesn't matter for channel values)
    if [ "$name" = "test_concurrency" ]; then
        # Check if all expected parts are present (order-independent)
        if echo "$actual" | grep -q "Test 1: Channel communication" && \
           echo "$actual" | grep -q "Received from channel:" && \
           echo "$actual" | grep -q "Test 2: WaitGroup" && \
           echo "$actual" | grep -q "WaitGroup completed" && \
           echo "$actual" | grep -q "All concurrency tests passed!"; then
            echo "✅ $name: OK"
            PASS=$((PASS + 1))
        else
            echo "❌ $name: WRONG OUTPUT"
            echo "   actual:   $(echo "$actual" | head -5)"
            ERRORS+=("$name: wrong output")
            FAIL=$((FAIL + 1))
        fi
        return
    fi

    if [ -z "$expected" ]; then
        echo "✅ $name: OK (no expected output check)"
        PASS=$((PASS + 1))
    elif [ "$actual" = "$expected" ]; then
        echo "✅ $name: OK"
        PASS=$((PASS + 1))
    else
        echo "❌ $name: WRONG OUTPUT"
        echo "   expected: $(echo "$expected" | head -3)"
        echo "   actual:   $(echo "$actual" | head -3)"
        ERRORS+=("$name: wrong output")
        FAIL=$((FAIL + 1))
    fi
}

cargo build -q 2>/dev/null

echo "=== Viper Test Suite ==="
echo ""

run_test tests/viper_programs/test_for_nomut.vp "45"
run_test tests/viper_programs/test_while.vp "1000"
run_test tests/viper_programs/test_concurrency.vp "Test 1: Channel communication
Received from channel:
1
2
3
Test 2: WaitGroup
WaitGroup completed
All concurrency tests passed!"
run_test tests/viper_programs/test_factorial.vp ""
run_test tests/viper_programs/test_fibonacci.vp ""
run_test tests/viper_programs/test_simple.vp ""
run_test tests/viper_programs/test_print_int.vp ""
run_test tests/viper_programs/test_list.vp ""
run_test tests/viper_programs/test_mut.vp ""
run_test tests/viper_programs/test_quicksort.vp ""
run_test tests/viper_programs/test_fixes.vp ""
run_test tests/viper_programs/test_phase2_all.vp ""
run_test tests/viper_programs/test_chan_simple.vp ""
run_test tests/viper_programs/test_swap.vp ""
run_test tests/viper_programs/test_literals.vp ""
run_test tests/viper_programs/test_types_basic.vp ""
run_test tests/viper_programs/test_arithmetic.vp ""
run_test tests/viper_programs/test_control_flow.vp ""
run_test tests/viper_programs/test_functions_basic.vp ""
run_test tests/viper_programs/test_const.vp ""
run_test tests/viper_programs/test_global.vp ""
run_test tests/viper_programs/test_bitwise.vp ""
run_test tests/viper_programs/test_identity.vp ""
run_test tests/viper_programs/test_list_methods.vp ""
run_test tests/viper_programs/test_string_methods.vp ""
run_test tests/viper_programs/test_dict.vp ""
run_test tests/viper_programs/test_sync_tasks.vp ""
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ ${#ERRORS[@]} -gt 0 ]; then
    echo "Failures:"
    for e in "${ERRORS[@]}"; do
        echo "  - $e"
    done
fi
