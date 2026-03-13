# Viper

Viper is a Python-inspired compiled language aimed at two goals at the same time:

- near-100% Python compatibility at the language level
- performance that can approach C on hot paths through strong compilation and runtime optimization

## Project Direction

The core rule for performance work is that Python-visible behavior comes first.

- default `int` behavior should remain Python-compatible and arbitrary-precision
- optimization should improve the implementation of Python-like semantics instead of replacing them with fixed-width shortcuts
- explicit low-level types such as `i64` are opt-in tools for users who want fixed-width behavior on purpose

In practice, that means Viper should try to be:

- familiar like Python
- predictable like Python
- fast through LLVM, native code generation, runtime specialization, and carefully chosen fast paths

## Development Principle

When there is tension between compatibility and speed, the preferred order is:

1. preserve Python-compatible semantics
2. optimize the common case internally
3. expose fixed-width or lower-level tradeoffs only as explicit opt-in choices

This keeps the language honest: fast because the implementation is good, not because Python semantics were quietly weakened.
