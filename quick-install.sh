#!/bin/bash
# Viper Quick Install Script
# For users who already have all dependencies installed
# Installs to $HOME/.local (no sudo required)

set -e

INSTALL_DIR="$HOME/.local"
VIPER_BIN="$INSTALL_DIR/bin"
VIPER_LIB="$INSTALL_DIR/lib/viper"
VIPER_INCLUDE="$INSTALL_DIR/include/viper"

echo "🐍 Viper Quick Install"
echo "======================"
echo ""

# Create directories
mkdir -p "$VIPER_BIN" "$VIPER_LIB" "$VIPER_INCLUDE"

# Clean previous installation
echo "Cleaning previous installation..."
rm -rf "$VIPER_BIN/viper" "$VIPER_LIB"/* "$VIPER_INCLUDE"/*

# Build compiler
echo "Building Viper compiler..."
cargo build --release

# Install binary
echo "Installing binary..."
cp "target/release/viper" "$VIPER_BIN/"
chmod +x "$VIPER_BIN/viper"

# Build runtime
echo "Building runtime library..."
cd runtime
make clean >/dev/null 2>&1 || true
make
cd ..

# Install runtime
echo "Installing runtime library..."
cp "runtime/obj/libviper.a" "$VIPER_LIB/"
cp "runtime/obj/"*.o "$VIPER_LIB/" 2>/dev/null || true
cp "runtime/include/"*.h "$VIPER_INCLUDE/"
cp "runtime/viper_stdlib.h" "$INSTALL_DIR/include/"

echo ""
echo "✅ Installation complete!"
echo ""
echo "Binary: $VIPER_BIN/viper"
echo ""
echo "Add to PATH:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
