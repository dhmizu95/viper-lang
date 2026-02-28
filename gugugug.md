Current Limitations:
Union return types that mix pointer and non-pointer types (e.g., -> i64 | str) require additional codegen work for proper tagged union representation
Runtime type narrowing (e.g., if isinstance(x, int)) is not yet implemented
Pattern matching on union types needs future implementation


