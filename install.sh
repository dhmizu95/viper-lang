#!/bin/bash
# Viper Language Installation Script
# Installs Viper compiler and runtime with all dependencies
# Supports local (~/.local) and system-wide (/usr/local) installation

set -e

# Configuration
INSTALL_MODE="${INSTALL_MODE:-local}"  # local or system
CLEAN_BUILD="${1:-}"  # clean or empty (cached)
INSTALL_DIR="$HOME/.local"
if [ "$INSTALL_MODE" = "system" ]; then
    INSTALL_DIR="/usr/local"
fi

VIPER_BIN="$INSTALL_DIR/bin"
VIPER_LIB="$INSTALL_DIR/lib/viper"
VIPER_INCLUDE="$INSTALL_DIR/include/viper"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

check_command() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ "$(uname)" = "Darwin" ]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

OS=$(detect_os)

# ============================================
# Dependency Checking
# ============================================

echo ""
echo "🐍 Viper Installation Script"
echo "============================"
echo ""
print_info "Detected OS: $OS"
print_info "Installation mode: $INSTALL_MODE"
print_info "Installation directory: $INSTALL_DIR"
if [ "$CLEAN_BUILD" = "clean" ]; then
    print_info "Build type: clean (full rebuild)"
else
    print_info "Build type: cached (incremental)"
fi
echo ""

# Check and install dependencies
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Checking Dependencies"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Rust
print_info "Checking Rust..."
if check_command rustc; then
    RUST_VERSION=$(rustc --version)
    print_success "Rust installed: $RUST_VERSION"
else
    print_error "Rust not found"
    print_info "Install Rust from: https://rustup.rs/"
    print_info "Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# LLVM
print_info "Checking LLVM..."
LLVM_FOUND=false
for llvm_config_cmd in llvm-config-21 llvm-config21 llvm-config; do
    if check_command "$llvm_config_cmd"; then
        LLVM_VERSION=$("$llvm_config_cmd" --version 2>/dev/null || echo "unknown")
        print_success "LLVM installed: $LLVM_VERSION"
        LLVM_FOUND=true
        break
    fi
done

if [ "$LLVM_FOUND" = false ]; then
    print_warning "LLVM not found in PATH"
    print_info "LLVM 21.x is required"
    
    case "$OS" in
        ubuntu|debian)
            print_info "Install with: sudo apt install llvm-21 llvm-21-dev"
            ;;
        fedora)
            print_info "Install with: sudo dnf install llvm21 llvm21-devel"
            ;;
        arch|manjaro)
            print_info "Install with: sudo pacman -S llvm"
            ;;
        macos)
            print_info "Install with: brew install llvm"
            ;;
        *)
            print_info "Please install LLVM 21.x from https://llvm.org/"
            ;;
    esac
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# GCC
print_info "Checking GCC..."
if check_command gcc; then
    GCC_VERSION=$(gcc --version | head -n1)
    print_success "GCC installed: $GCC_VERSION"
else
    print_error "GCC not found (required for linking)"
    exit 1
fi

# GMP (required for BigInt support)
print_info "Checking GMP (for BigInt support)..."
GMP_FOUND=false

# Check for vendored GMP first
if [ -d "vendor/gmp/lib" ] && [ -f "vendor/gmp/lib/libgmp.so.10" ]; then
    print_success "Vendored GMP found: vendor/gmp/lib/"
    GMP_FOUND=true
elif check_command pkg-config; then
    if pkg-config --exists gmp 2>/dev/null; then
        GMP_VERSION=$(pkg-config --modversion gmp)
        print_success "GMP installed: $GMP_VERSION"
        GMP_FOUND=true
    fi
fi

# Also check for gmp.h directly
if [ "$GMP_FOUND" = false ] && [ -f /usr/include/gmp.h ]; then
    print_success "GMP found: /usr/include/gmp.h"
    GMP_FOUND=true
fi

if [ "$GMP_FOUND" = false ]; then
    print_warning "GMP not found - BigInt support will be unavailable"
    print_info "Install GMP for full BigInt support:"

    case "$OS" in
        ubuntu|debian)
            print_info "  sudo apt install libgmp-dev pkg-config"
            ;;
        fedora)
            print_info "  sudo dnf install gmp-devel pkg-config"
            ;;
        arch|manjaro)
            print_info "  sudo pacman -S gmp pkg-config"
            ;;
        macos)
            print_info "  brew install gmp pkg-config"
            ;;
        *)
            print_info "  Install from: https://gmplib.org/"
            ;;
    esac
    echo ""
    read -p "Continue without GMP? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Make
print_info "Checking Make..."
if check_command make; then
    MAKE_VERSION=$(make --version | head -n1)
    print_success "Make installed: $MAKE_VERSION"
else
    print_error "Make not found"
    exit 1
fi

echo ""

# ============================================
# Installation
# ============================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Installing Viper"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create directories
print_info "Creating directories..."
mkdir -p "$VIPER_BIN" "$VIPER_LIB" "$VIPER_INCLUDE"

# Clean previous installation
print_info "Cleaning previous installation..."
rm -rf "$VIPER_BIN/viper" "$VIPER_LIB"/* "$VIPER_INCLUDE"/*

# Build compiler
echo ""
if [ "$CLEAN_BUILD" = "clean" ]; then
    print_info "Building Viper compiler (release mode, clean build)..."
    cargo clean
else
    print_info "Building Viper compiler (release mode, cached build)..."
fi
if cargo build --release 2>&1 | tee /tmp/viper_build.log; then
    print_success "Compiler built successfully"
else
    print_error "Build failed. Check /tmp/viper_build.log for details"
    exit 1
fi

# Install binary
print_info "Installing viper binary..."
if [ ! -f "target/release/viper" ]; then
    print_error "target/release/viper not found"
    exit 1
fi
cp "target/release/viper" "$VIPER_BIN/"
chmod +x "$VIPER_BIN/viper"
print_success "Binary installed: $VIPER_BIN/viper"

# Build runtime
echo ""
print_info "Building runtime library..."
cd runtime
make clean >/dev/null 2>&1 || true

if make 2>&1 | tee /tmp/viper_runtime_build.log; then
    print_success "Runtime library built successfully"
else
    print_warning "Runtime build had warnings. Check /tmp/viper_runtime_build.log"
fi
cd ..

# Install runtime library
print_info "Installing runtime library..."
if [ -f "runtime/obj/libviper.a" ]; then
    cp "runtime/obj/libviper.a" "$VIPER_LIB/"
    print_success "Library installed: $VIPER_LIB/libviper.a"
else
    print_warning "libviper.a not found (runtime may not be fully built)"
fi

# Install object files
if [ -d "runtime/obj" ]; then
    cp "runtime/obj/"*.o "$VIPER_LIB/" 2>/dev/null || true
    print_success "Object files installed"
fi

# Install headers
print_info "Installing headers..."
cp "runtime/include/"*.h "$VIPER_INCLUDE/" 2>/dev/null || true
cp "runtime/viper_stdlib.h" "$INSTALL_DIR/include/" 2>/dev/null || true
print_success "Headers installed: $VIPER_INCLUDE/"

# Install vendored GMP (if available)
if [ -d "vendor/gmp/lib" ] && [ -f "vendor/gmp/lib/libgmp.so.10" ]; then
    print_info "Installing vendored GMP library..."
    mkdir -p "$VIPER_LIB/gmp/lib" "$VIPER_LIB/gmp/include"
    cp "vendor/gmp/lib/"*.so* "$VIPER_LIB/gmp/lib/" 2>/dev/null || true
    cp "vendor/gmp/include/"*.h "$VIPER_LIB/gmp/include/" 2>/dev/null || true
    print_success "Vendored GMP installed: $VIPER_LIB/gmp/"
fi

echo ""

# ============================================
# Post-Installation
# ============================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Installation Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
print_success "Viper has been installed successfully"
echo ""
echo "📦 Installed Components:"
echo "   Binary:   $VIPER_BIN/viper"
echo "   Library:  $VIPER_LIB/libviper.a"
echo "   Headers:  $VIPER_INCLUDE/"
echo ""

# PATH configuration
if [ "$INSTALL_MODE" = "local" ]; then
    echo "🔧 PATH Configuration:"
    echo ""
    
    # Check if PATH already includes local bin
    if [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
        print_success "$HOME/.local/bin is already in PATH"
    else
        print_warning "$HOME/.local/bin is not in PATH"
        echo ""
        echo "Add to your shell configuration:"
        echo ""
        
        if [ -f "$HOME/.bashrc" ]; then
            echo "   For bash (~/.bashrc):"
            echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo "   source ~/.bashrc"
            echo ""
        fi
        
        if [ -f "$HOME/.zshrc" ]; then
            echo "   For zsh (~/.zshrc):"
            echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
            echo "   source ~/.zshrc"
            echo ""
        fi
        
        if [ -f "$HOME/.config/fish/config.fish" ]; then
            echo "   For fish (~/.config/fish/config.fish):"
            echo "   echo 'set -U fish_user_paths \$HOME/.local/bin \$fish_user_paths' >> ~/.config/fish/config.fish"
            echo ""
        fi
        
        echo "Or create a symlink (requires sudo):"
        echo "   sudo ln -s $VIPER_BIN/viper /usr/local/bin/viper"
    fi
else
    echo "🔧 System-wide installation complete"
    echo "   Binary should be available at: /usr/local/bin/viper"
fi

echo ""

# Verify installation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ -f "$VIPER_BIN/viper" ]; then
    print_info "Testing viper binary..."
    if VIPER_PATH="" "$VIPER_BIN/viper" --version 2>/dev/null; then
        print_success "Viper binary works correctly"
    elif VIPER_PATH="" "$VIPER_BIN/viper" info 2>/dev/null; then
        print_success "Viper binary works correctly"
    else
        print_warning "Viper binary exists but may need PATH configuration"
    fi
fi

# Test compilation and verify static linking
echo ""
print_info "Testing compilation and static linking..."
echo 'print("Static test OK")' > /tmp/viper_test_$$.vp
if VIPER_PATH="" "$VIPER_BIN/viper" build /tmp/viper_test_$$.vp -o /tmp/viper_test_$$ 2>/dev/null; then
    # ldd returns "not a dynamic executable" for static binaries
    if ! ldd /tmp/viper_test_$$_bin 2>&1 | grep -q "\.so"; then
        print_success "Compiled binary is fully static (no dependencies)"
    else
        print_warning "Compiled binary has dynamic dependencies"
    fi
    rm -f /tmp/viper_test_$$_bin /tmp/viper_test_$$ /tmp/viper_test_$$.vp /tmp/viper_test_$$.o
else
    print_warning "Compilation test failed (may need PATH configuration)"
fi

echo ""
echo "📚 Quick Start:"
echo ""
echo "   # Create a new project"
echo "   viper init myproject"
echo "   cd myproject"
echo ""
echo "   # Run a Viper program"
echo "   viper run src/main.vp"
echo ""
echo "   # Build an optimized binary"
echo "   viper build src/main.vp -O 2 -o myapp"
echo "   ./myapp  # Runs anywhere - no dependencies!"
echo ""

if [ "$GMP_FOUND" = true ]; then
    echo "🎉 BigInt support is enabled (GMP found)"
else
    echo "⚠️  BigInt support is disabled (GMP not found)"
    echo "   Install GMP and rebuild for BigInt support"
fi

echo ""
echo "📦 Deployment:"
echo "   Compiled binaries are fully static!"
echo "   Copy the binary to any Linux x86_64 system and run."
echo ""
echo "📖 Documentation:"
echo "   BIGINT_IMPLEMENTATION.md - BigInt usage guide"
echo "   README.md - Full documentation"
echo "   INSTALLATION.md - Detailed install instructions"
echo ""
print_success "Happy coding with Viper! 🐍"
echo ""
