#!/bin/bash
# Test Viper AOT compilation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VIPER_BIN="$PROJECT_ROOT/target/release/viper"

echo "=== Testing Viper AOT Compilation ==="
echo

# Test 1: Fibonacci AOT
echo "Test 1: Fibonacci AOT (-O0)"
if "$VIPER_BIN" build -O0 "$SCRIPT_DIR/viper/01_fibonacci.vp" -o /tmp/fib_aot 2>&1 | tail -5; then
    echo "  Build: Success"
    if [ -f /tmp/fib_aot_bin ]; then
        echo "  Running AOT binary..."
        /tmp/fib_aot_bin 2>&1
        echo "  ✅ AOT Fibonacci: PASS"
    else
        echo "  ❌ AOT binary not found"
    fi
else
    echo "  ❌ AOT build failed"
fi
echo

# Test 2: Prime Sieve AOT
echo "Test 2: Prime Sieve AOT (-O0)"
if "$VIPER_BIN" build -O0 "$SCRIPT_DIR/viper/02_prime_sieve.vp" -o /tmp/sieve_aot 2>&1 | tail -5; then
    echo "  Build: Success"
    if [ -f /tmp/sieve_aot_bin ]; then
        echo "  Running AOT binary..."
        /tmp/sieve_aot_bin 2>&1
        echo "  ✅ AOT Prime Sieve: PASS"
    else
        echo "  ❌ AOT binary not found"
    fi
else
    echo "  ❌ AOT build failed"
fi
echo

# Cleanup
rm -f /tmp/fib_aot* /tmp/sieve_aot* /tmp/*.o /tmp/*.bc /tmp/*.opt.bc

echo "=== AOT Test Complete ==="
