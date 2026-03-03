#!/bin/bash

# Test 01_data_types in AOT mode only (JIT is currently broken)

VIPER=./target/debug/viper
TEST_DIR=tests/02_python_compat/01_data_types

echo "=== Testing 01_data_types: AOT Mode ==="
echo ""

# Build first
cargo build -q 2>/dev/null

PASS=0
FAIL=0

for file in $TEST_DIR/*.vp; do
    name=$(basename "$file" .vp)
    
    echo "=== $name ==="
    
    # AOT mode
    if $VIPER build "$file" >/dev/null 2>&1; then
        bin="${name}_vp_bin"
        if [ -f "$bin" ]; then
            output=$(timeout 2 ./"$bin" 2>&1)
            rc=$?
            if [ $rc -eq 0 ]; then
                echo "✅ AOT: PASS"
                echo "Output:"
                echo "$output"
                PASS=$((PASS + 1))
            else
                echo "❌ AOT: RUNTIME ERROR (exit $rc)"
                echo "Output: $(echo "$output" | head -3)"
                FAIL=$((FAIL + 1))
            fi
            rm -f "$bin"
        else
            echo "❌ AOT: BINARY NOT FOUND"
            FAIL=$((FAIL + 1))
        fi
    else
        echo "❌ AOT: COMPILE ERROR"
        FAIL=$((FAIL + 1))
    fi
    
    echo ""
done

echo "=== Summary ==="
echo "AOT: $PASS passed, $FAIL failed"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✅ All AOT tests passed!"
    exit 0
else
    echo "❌ Some AOT tests failed"
    exit 1
fi
