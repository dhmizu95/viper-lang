#!/bin/bash
# Prime Sieve Benchmark Comparison
# Compares C, Rust, Go, Viper JIT, and Viper AOT

cd "$(dirname "$0")"

LIMIT=1000000

echo "========================================"
echo "Prime Sieve Benchmark Comparison"
echo "Limit: $LIMIT"
echo "========================================"
echo ""

# C
echo "=== C ==="
if [ -f ./sieve_c ]; then
    time ./sieve_c
else
    echo "sieve_c not found"
fi
echo ""

# Rust
echo "=== Rust ==="
if [ -f ./sieve_rs ]; then
    time ./sieve_rs
else
    echo "sieve_rs not found"
fi
echo ""

# Go
echo "=== Go ==="
if [ -f ./sieve_go ]; then
    time ./sieve_go
else
    echo "sieve_go not found"
fi
echo ""

# Viper AOT
echo "=== Viper AOT ==="
if [ -f ./sieve_vp_aot_bin ]; then
    time ./sieve_vp_aot_bin
else
    echo "sieve_vp_aot_bin not found"
fi
echo ""

# Viper JIT
echo "=== Viper JIT ==="
if [ -f ./sieve.vp ]; then
    time /home/stl/viper-lang/target/release/viper run sieve.vp
else
    echo "sieve.vp not found"
fi
echo ""

echo "========================================"
echo "Comparison complete!"
echo "========================================"
