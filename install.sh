#!/bin/bash
# Viper System-Wide Installation Script
# Installs to $HOME/.local (no sudo required)

set -e

INSTALL_DIR="$HOME/.local"
VIPER_BIN="$INSTALL_DIR/bin"
VIPER_LIB="$INSTALL_DIR/lib/viper"
VIPER_INCLUDE="$INSTALL_DIR/include/viper"

echo "🐍 Viper Installation Script"
echo "============================"
echo ""

# Create directories
echo "Creating directories..."
mkdir -p "$VIPER_BIN" "$VIPER_LIB" "$VIPER_INCLUDE"

# Build release binary if it doesn't exist
if [ ! -f "target/release/viper" ]; then
    echo "Building release binary..."
    cargo build --release
fi

# Copy binary
echo "Installing viper binary..."
if [ ! -f "target/release/viper" ]; then
    echo "❌ Error: target/release/viper not found after build"
    exit 1
fi
cp "target/release/viper" "$VIPER_BIN/"

# Copy runtime library
echo "Installing runtime library..."
if [ ! -f "runtime/obj/libviper.a" ]; then
    echo "Building runtime library..."
    cd runtime && make && cd ..
fi
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
