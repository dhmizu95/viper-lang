#!/bin/bash
# Viper System-Wide Installation Script
# Installs to $HOME/.local (no sudo required)
# Always performs a clean install

set -e

INSTALL_DIR="$HOME/.local"
VIPER_BIN="$INSTALL_DIR/bin"
VIPER_LIB="$INSTALL_DIR/lib/viper"
VIPER_INCLUDE="$INSTALL_DIR/include/viper"

echo "🐍 Viper Installation Script"
echo "============================"
echo ""

# Clean previous installation
echo "Cleaning previous installation..."
rm -rf "$VIPER_BIN/viper" "$VIPER_LIB"/* "$VIPER_INCLUDE"/*

# Create directories
echo "Creating directories..."
mkdir -p "$VIPER_BIN" "$VIPER_LIB" "$VIPER_INCLUDE"

# Clean and build release binary
echo "Building release binary (clean build)..."
cargo clean
cargo build --release

# Copy binary
echo "Installing viper binary..."
if [ ! -f "target/release/viper" ]; then
    echo "❌ Error: target/release/viper not found after build"
    exit 1
fi
cp "target/release/viper" "$VIPER_BIN/"

# Clean and build runtime library
echo "Building runtime library (clean build)..."
cd runtime
make clean
make
cd ..

# Copy runtime library
echo "Installing runtime library..."
if [ ! -f "runtime/obj/libviper.a" ]; then
    echo "❌ Error: runtime/obj/libviper.a not found"
    exit 1
fi
cp "runtime/obj/libviper.a" "$VIPER_LIB/"
cp "runtime/obj/"*.o "$VIPER_LIB/"

# Copy headers
echo "Installing headers..."
cp "runtime/include/"*.h "$VIPER_INCLUDE/"
cp "runtime/viper_stdlib.h" "$INSTALL_DIR/include/"

echo ""
echo "✅ Installation complete!"
echo ""
echo "Binary: $VIPER_BIN/viper"
echo "Library: $VIPER_LIB/libviper.a"
echo "Headers: $VIPER_INCLUDE/"
echo ""
echo "Add to your PATH (add to ~/.bashrc or ~/.zshrc):"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Or create symlinks:"
echo "  ln -s $VIPER_BIN/viper /usr/local/bin/viper  (requires sudo)"
