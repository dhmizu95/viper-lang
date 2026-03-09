#!/bin/bash
# Test Plan Execution Script for Viper Language

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_section() {
    echo -e "\n${YELLOW}>> $1${NC}\n"
}

run_cargo_test() {
    local category=$1
    echo -e "${YELLOW}Running: cargo test $category${NC}"
    if cargo test "$category" --quiet 2>&1; then
        echo -e "${GREEN}✓ $category passed${NC}"
    else
        echo -e "${RED}✗ $category failed${NC}"
    fi
}

print_header "Viper Language Test Plan Execution"

print_section "1. Lexer Tests"
run_cargo_test "lexer"

print_section "2. Parser Tests"
run_cargo_test "parser"

print_section "3. AST Tests"
run_cargo_test "ast"

print_section "4. Semantic Analysis Tests"
run_cargo_test "semantic"

print_section "5. Utils Tests"
run_cargo_test "utils"

print_section "6. Integration Tests"
run_cargo_test "integration"

print_header "Test Plan Execution Complete"
