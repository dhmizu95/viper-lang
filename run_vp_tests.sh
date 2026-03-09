#!/bin/bash

cd /home/user/viper-lang

passed=0
failed=0
crashed=0

for f in tests/*.vp; do
    result=$(timeout 10 ./target/debug/viper run "$f" 2>&1)
    if echo "$result" | grep -q "✅ Execution complete"; then
        passed=$((passed+1))
        echo "✅ PASS: $f"
    elif echo "$result" | grep -q "Error:"; then
        failed=$((failed+1))
        err_msg=$(echo "$result" | grep "Error:" | head -1)
        echo "❌ FAIL: $f - $err_msg"
    else
        crashed=$((crashed+1))
        echo "❌ CRASH: $f"
    fi
done

echo ""
echo "=== Summary ==="
echo "Passed: $passed"
echo "Failed: $failed"
echo "Crashed: $crashed"
echo "Total: $((passed + failed + crashed))"
