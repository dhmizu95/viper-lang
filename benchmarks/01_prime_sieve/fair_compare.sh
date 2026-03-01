#!/bin/bash
# Fair Prime Sieve Comparison - All languages at N=1000000

cd "$(dirname "$0")"

N=1000000

echo "========================================"
echo "Prime Sieve Fair Comparison (N=$N)"
echo "========================================"
echo ""

# Create temp files for each language with N=1000000

# C
echo "C:"
sed "s/10000000/$N/" sieve.c > sieve_1m.c
gcc -O3 -o sieve_1m_c sieve_1m.c -lm
time ./sieve_1m_c 2>&1 | grep -E "(Primes|Time)"
rm -f sieve_1m.c sieve_1m_c

# Rust  
echo ""
echo "Rust:"
sed 's/10000000/1000000/' sieve.rs > sieve_1m.rs
rustc -O -o sieve_1m_rs sieve_1m.rs
time ./sieve_1m_rs 2>&1 | grep -E "(Primes|Time)"
rm -f sieve_1m.rs sieve_1m_rs

# Go
echo ""
echo "Go:"
sed 's/10000000/1000000/' sieve.go > sieve_1m.go
go build -o sieve_1m_go sieve_1m.go
time ./sieve_1m_go 2>&1 | grep -E "(Primes|Time)"
rm -f sieve_1m.go sieve_1m_go

# Viper AOT
echo ""
echo "Viper AOT:"
cat > sieve_1m.vp << 'EOF'
def sieve(n: i64) -> i64:
    is_prime = []
    i = 0
    while i <= n:
        is_prime.append(1)
        i = i + 1
    is_prime[0] = 0
    is_prime[1] = 0
    i = 2
    while i * i <= n:
        if is_prime[i] == 1:
            j = i * i
            while j <= n:
                is_prime[j] = 0
                j = j + i
        i = i + 1
    count = 0
    i = 2
    while i <= n:
        if is_prime[i] == 1:
            count = count + 1
        i = i + 1
    return count

def main():
    n = 1000000
    count = sieve(n)
    print("Primes found:", count)
EOF
/home/stl/viper-lang/target/release/viper build sieve_1m.vp -o sieve_1m_vp_aot 2>&1 | tail -1
time ./sieve_1m_vp_aot_bin 2>&1 | grep -v "^$"
rm -f sieve_1m.vp sieve_1m_vp_aot sieve_1m_vp_aot.o sieve_1m_vp_aot_bin

# Viper JIT
echo ""
echo "Viper JIT:"
time /home/stl/viper-lang/target/release/viper run sieve_1m.vp 2>&1 | grep -E "(Primes|complete)"
rm -f sieve_1m.vp

echo ""
echo "========================================"
echo "Comparison complete!"
echo "========================================"
