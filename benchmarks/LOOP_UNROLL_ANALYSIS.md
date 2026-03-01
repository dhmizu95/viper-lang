# Loop Unrolling Analysis

## Attempted Implementation

Tried manual loop unrolling with factor of 4 for while loops.

### Result: **Slower** (1.27s vs 1.04s)

**Reason**: The manual unrolling added extra condition checks that outweighed the benefits.

## LLVM's Built-in Unrolling

LLVM's `-O2` optimization already includes sophisticated loop unrolling:
- Analyzes loop trip counts
- Considers code size impact
- Uses profile-guided optimization when available

## Best Configuration

The **branch prediction hints** (`__builtin_expect`) provided the best improvement:

```c
if (__builtin_expect(idx < 0, 0)) idx = vec->length + idx;
if (__builtin_expect(vec->length >= vec->capacity, 0)) {
    vp_bitvec_grow(vec);
}
```

This tells the CPU branch predictor that:
- Negative indices are rare (0% probability)
- Vector growth is rare (0% probability)

## Performance Summary (100M Prime Sieve)

| Optimization | Time | Improvement |
|--------------|------|-------------|
| Baseline | 1,349ms | - |
| + LTO | 1,282ms | 5% |
| + Branch Prediction | 1,043ms | 23% total |
| + Manual Unroll | 1,271ms | **Regression** |
| LLVM Auto-Unroll | ~1,200ms | Varies |

## Conclusion

**Branch prediction is the winner** for this workload. Manual loop unrolling:
- Increases code size
- Adds extra condition checks
- May interfere with LLVM's own optimizations

**Recommendation**: Trust LLVM's optimizer for loop unrolling, focus on:
1. ✅ Branch prediction (done)
2. ✅ LTO (done)
3. ⏳ ARC optimization (next)
4. ⏳ Cache-friendly data layouts
