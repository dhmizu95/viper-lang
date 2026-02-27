#!/bin/bash
# Viper Uninstall Script
# Removes Viper installation from the system

set -e

INSTALL_MODE="${INSTALL_MODE:-local}"  # local or system
INSTALL_DIR="$HOME/.local"
if [ "$INSTALL_MODE" = "system" ]; then
    INSTALL_DIR="/usr/local"
fi

VIPER_BIN="$INSTALL_DIR/bin"
VIPER_LIB="$INSTALL_DIR/lib/viper"
VIPER_INCLUDE="$INSTALL_DIR/include/viper"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

echo ""
echo "🐍 Viper Uninstall Script"
echo "========================="
echo ""
print_info "Installation directory: $INSTALL_DIR"
echo ""

read -p "Are you sure you want to uninstall Viper? (y/N): " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstall cancelled"
    exit 0
fi

echo ""
print_info "Removing Viper components..."

# Remove binary
if [ -f "$VIPER_BIN/viper" ]; then
    rm "$VIPER_BIN/viper"
    print_success "Removed: $VIPER_BIN/viper"
else
    print_info "Binary not found: $VIPER_BIN/viper"
fi

# Remove runtime library
if [ -d "$VIPER_LIB" ]; then
    rm -rf "$VIPER_LIB"
    print_success "Removed: $VIPER_LIB"
else
    print_info "Library directory not found: $VIPER_LIB"
fi

# Remove headers
if [ -d "$VIPER_INCLUDE" ]; then
    rm -rf "$VIPER_INCLUDE"
    print_success "Removed: $VIPER_INCLUDE"
else
    print_info "Headers directory not found: $VIPER_INCLUDE"
fi

# Remove standalone header
if [ -f "$INSTALL_DIR/include/viper_stdlib.h" ]; then
    rm "$INSTALL_DIR/include/viper_stdlib.h"
    print_success "Removed: $INSTALL_DIR/include/viper_stdlib.h"
fi

# Clean build artifacts
echo ""
print_info "Cleaning build artifacts..."
if [ -d "target" ]; then
    rm -rf target
    print_success "Removed: target/"
fi

if [ -d "runtime/obj" ]; then
    rm -rf runtime/obj
    print_success "Removed: runtime/obj/"
fi

if [ -d "runtime/lib" ]; then
    rm -rf runtime/lib
    print_success "Removed: runtime/lib/"
fi

echo ""
print_success "Viper has been uninstalled"
echo ""
print_info "Optional: Remove source code"
echo "  rm -rf viper-lang/"
echo ""
print_info "Optional: Remove PATH configuration from your shell config"
echo "  Remove this line from ~/.bashrc, ~/.zshrc, etc.:"
echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
