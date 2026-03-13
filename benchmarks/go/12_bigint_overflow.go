// BigInt Overflow Path Benchmark - Go Implementation

package main

import (
	"fmt"
	"math/big"
)

func main() {
	value := new(big.Int).Lsh(big.NewInt(1), 100)
	addend := big.NewInt(123456789)
	subtrahend := big.NewInt(98765432)
	modulus := big.NewInt(97)
	checksum := int64(0)
	tmp := new(big.Int)

	for i := 0; i < 200000; i++ {
		value.Add(value, addend)
		value.Sub(value, subtrahend)
		tmp.Mod(value, modulus)
		checksum += tmp.Int64()
	}

	fmt.Printf("bigint overflow checksum: %d\n", checksum)
}
