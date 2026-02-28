# Viper Installation Guide

This guide covers installing the Viper compiler and runtime library on various platforms.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/viper-lang/viper.git
cd viper-lang

# Run the installation script
./install.sh

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Verify installation
viper info
```

## What Gets Installed

| Component | Description | Size | Dependencies |
|-----------|-------------|------|--------------|
| **Viper Compiler** (`viper`) | Compiles `.vp` to native binaries | 4.2M | LLVM 20, libc, libstdc++ |
| **Runtime Library** (`libviper.a`) | C runtime for compiled programs | 1.4M | GMP (for BigInt) |
| **Compiled Programs** | Your AOT-compiled binaries | ~800K | **None** (fully static) |

> **Note:** Compiled Viper programs are **fully static** - they require no runtime libraries or dependencies. Just copy the binary and run!

---

## Installation Scripts

### Full Installation (Recommended)

```bash
./install.sh
```

This script:
- ✅ Checks all dependencies
- ✅ Provides installation instructions for missing dependencies
- ✅ Builds compiler in release mode
- ✅ Builds and installs runtime library
- ✅ Installs headers for development

Options:
```bash
# System-wide installation (requires sudo)
INSTALL_MODE=system sudo ./install.sh

# Local installation (default, no sudo required)
./install.sh
```

### Quick Installation

```bash
./quick-install.sh
```

Use this if you already have all dependencies installed. Skips dependency checks.

### Uninstallation

```bash
./uninstall.sh
```

Removes all Viper components from your system.

---

## Dependencies

### Required

| Dependency | Version | Purpose |
|------------|---------|---------|
| Rust | 1.70+ | Compiler implementation |
| LLVM | 20.x | IR generation and optimization |
| GCC | Any recent | Linking AOT binaries |
| Make | 3.81+ | Building runtime |

### Optional (for BigInt support)

| Dependency | Version | Purpose |
|------------|---------|---------|
| GMP | 6.0+ | Arbitrary-precision integers |
| pkg-config | 0.29+ | Finding GMP |

---

## Platform-Specific Instructions

### Ubuntu / Debian

```bash
# Update package list
sudo apt update

# Install dependencies
sudo apt install -y \
    curl \
    build-essential \
    llvm-20 \
    llvm-20-dev \
    libgmp-dev \
    pkg-config

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Viper
./install.sh

# Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Fedora / RHEL

```bash
# Install dependencies
sudo dnf install -y \
    curl \
    gcc \
    make \
    llvm20 \
    llvm20-devel \
    gmp-devel \
    pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Viper
./install.sh

# Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Arch Linux / Manjaro

```bash
# Install dependencies
sudo pacman -S \
    base-devel \
    llvm \
    gmp \
    pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Viper
./install.sh

# Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### macOS

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install \
    rust \
    llvm \
    gmp \
    pkg-config

# Install Viper
./install.sh

# Add to PATH (for Apple Silicon)
echo 'export PATH="/opt/homebrew/bin:$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# For Intel Macs:
# echo 'export PATH="/usr/local/bin:$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

### Windows (WSL2)

```bash
# In WSL2 Ubuntu terminal
sudo apt update
sudo apt install -y \
    curl \
    build-essential \
    llvm-20 \
    llvm-20-dev \
    libgmp-dev \
    pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Viper
./install.sh

# Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## Manual Installation

If you prefer manual installation:

### 1. Build Compiler

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release
```

### 2. Build Runtime

```bash
cd runtime
make          # Release build
make debug    # Debug build
make profile  # Profiling build
```

### 3. Install Files

```bash
# Binary
cp target/release/viper ~/.local/bin/

# Runtime library
mkdir -p ~/.local/lib/viper
cp runtime/obj/libviper.a ~/.local/lib/viper/

# Headers
mkdir -p ~/.local/include/viper
cp runtime/include/*.h ~/.local/include/viper/
```

### 4. Configure PATH

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## Verification

After installation, verify everything works:

```bash
# Check viper binary
viper --version
viper info

# Test compilation
echo 'print("Hello, Viper!")' > test.vp
viper build test.vp -o hello

# Verify binary is fully static (no dependencies)
ldd hello
# Output: "not a dynamic executable"

# Run the binary
./hello

# Clean up
rm test.vp hello
```

---

## Troubleshooting

### "LLVM not found"

```bash
# Check LLVM installation
llvm-config --version

# If not found, add to PATH
export PATH="/usr/lib/llvm-20/bin:$PATH"

# Or set LLVM_SYS_201_PREFIX
export LLVM_SYS_201_PREFIX=/usr/lib/llvm-20
```

### "GMP not found"

```bash
# Install GMP
# Ubuntu/Debian: sudo apt install libgmp-dev
# Fedora: sudo dnf install gmp-devel
# macOS: brew install gmp

# Verify
pkg-config --libs --cflags gmp
```

### "Runtime object files not found"

```bash
cd runtime
make clean
make
cd ..
```

### "Permission denied" during installation

For local installation (no sudo):
```bash
./install.sh
```

For system-wide installation:
```bash
sudo INSTALL_MODE=system ./install.sh
```

### Build fails with out of memory

```bash
# Reduce parallelism
export CARGO_BUILD_JOBS=1
cargo build --release
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `INSTALL_MODE` | Installation mode (local/system) | local |
| `CARGO_BUILD_JOBS` | Parallel compilation jobs | Auto |
| `LLVM_SYS_201_PREFIX` | LLVM 20 installation path | Auto-detect |
| `VIPER_PATH` | Additional module search path | Empty |

---

## Uninstallation

```bash
# Using script
./uninstall.sh

# Manual removal
rm -rf ~/.local/bin/viper
rm -rf ~/.local/lib/viper
rm -rf ~/.local/include/viper
rm -rf ~/.cargo/registry/src/*/viper-lang-*
```

---

## Next Steps

After successful installation:

1. **Read the documentation**
   - `README.md` - Compiler usage
   - `BIGINT_IMPLEMENTATION.md` - BigInt guide
   - `CORE_LANGUAGE_FEATURES.md` - Language features

2. **Try examples**
   ```bash
   viper run tests/bigint_test.vp
   ```

3. **Create your first project**
   ```bash
   viper init myproject
   cd myproject
   viper run src/main.vp
   ```

---

## Support

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Documentation**: See `docs/` directory
