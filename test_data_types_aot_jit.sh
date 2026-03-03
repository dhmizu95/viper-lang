#!/bin/bash

# Test 01_data_types in both AOT and JIT modes

VIPER=./target/debug/viper
TEST_DIR=tests/02_python_compat/01_data_types
PASS_AOT=0
FAIL_AOT=0
PASS_JIT=0
FAIL_JIT=0

echo "=== Testing 01_data_types: AOT vs JIT ==="
echo ""

# Build first
cargo build -q 2>/dev/null

for file in $(find $TEST_DIR -name "*.vp" | sort); do
    name=$(basename "$file" .vp)
    
    echo "Testing: $name"
    
    # AOT mode
    if $VIPER build "$file" >/dev/null 2>&1; then
        bin="${name}_vp_bin"
        if [ -f "$bin" ]; then
            aot_output=$(timeout 5 ./"$bin" 2>&1)
            aot_rc=$?
            if [ $aot_rc -eq 0 ]; then
                echo "  ✅ AOT: PASS"
                PASS_AOT=$((PASS_AOT + 1))
            else
                echo "  ❌ AOT: RUNTIME ERROR (exit $aot_rc)"
                FAIL_AOT=$((FAIL_AOT + 1))
            fi
        else
            echo "  ❌ AOT: BINARY NOT FOUND"
            FAIL_AOT=$((FAIL_AOT + 1))
        fi
    else
        echo "  ❌ AOT: COMPILE ERROR"
        FAIL_AOT=$((FAIL_AOT + 1))
    fi
    
    # JIT mode
    jit_output=$($VIPER run "$file" 2>&1)
    jit_rc=$?
    if [ $jit_rc -eq 0 ]; then
        echo "  ✅ JIT: PASS"
        PASS_JIT=$((PASS_JIT + 1))
    else
        echo "  ❌ JIT: ERROR (exit $jit_rc)"
        FAIL_JIT=$((FAIL_JIT + 1))
    fi
    
    # Compare outputs
    if [ $aot_rc -eq 0 ] && [ $jit_rc -eq 0 ]; then
        if [ "$aot_output" = "$jit_output" ]; then
            echo "  ✓ Outputs match"
        else
            echo "  ⚠ Outputs differ:"
            echo "    AOT: $(echo "$aot_output" | head -1)"
            echo "    JIT: $(echo "$jit_output" | head -1)"
        fi
    fi
    
    echo ""
done

echo "=== Summary ==="
echo "AOT: $PASS_AOT passed, $FAIL_AOT failed"
echo "JIT: $PASS_JIT passed, $FAIL_JIT failed"
echo ""

# Cleanup AOT binaries
for file in $(find $TEST_DIR -name "*.vp"); do
    name=$(basename "$file" .vp)
    rm -f "${name}_vp_bin"
done

if [ $FAIL_AOT -eq 0 ] && [ $FAIL_JIT -eq 0 ]; then
    echo "✅ All tests passed in both modes!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
