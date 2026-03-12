# Build Warning Messages for Viper Compiler

This document outlines the implementation of warning messages during the Viper compiler build process.

## Overview

Adding warning messages to the build process helps developers identify potential issues early and ensures build prerequisites are met.

## Implementation

### 1. Build Script Warnings (`build.rs`)

The build script now emits warnings for:

- **Build identification**: Shows compiler version being built
- **GMP dependency**: Reminds users to ensure GMP library is installed
- **Runtime directory**: Warns if runtime/obj directory doesn't exist
- **GMP vendor library**: Warns if vendor/gmp/lib directory doesn't exist

### 2. Makefile Targets

New targets for warning-aware builds:

| Target | Description |
|--------|-------------|
| `make build-warn` | Build with extra warnings enabled |
| `make check-warn` | Check with extra warnings enabled |
| `make clippy` | Run clippy with pedantic warnings |
| `make clean-warn` | Clean and rebuild with warnings |

### 3. Usage Examples

```bash
# Standard build (shows basic warnings)
cargo build

# Build with extra warnings
make build-warn

# Run clippy for additional linting
make clippy

# Clean rebuild with all warnings
make clean-warn
```

## Warning Categories

### Build Prerequisite Warnings

These warnings appear when required dependencies are missing:

```
warning: Runtime directory not found: /path/to/runtime/obj
warning: Run 'make runtime' to build the runtime library
warning: GMP vendor library not found: /path/to/vendor/gmp/lib
warning: Ensure GMP is installed or run the setup script
```

### Compiler Warnings

Rust compiler warnings for code quality:

- Unused variables
- Unused mutable references
- Dead code
- Deprecated APIs
- Clippy lints

## Benefits

1. **Early detection**: Catch missing dependencies before runtime failures
2. **Developer guidance**: Clear instructions on how to fix issues
3. **Code quality**: Consistent warning levels across builds
4. **Build verification**: Confirm all components are properly configured

## Future Enhancements

- [ ] Add warning summary at end of build
- [ ] Integrate with CI/CD for warning thresholds
- [ ] Add optional `-Werror` flag to treat warnings as errors
- [ ] Document common warning resolutions in troubleshooting guide
