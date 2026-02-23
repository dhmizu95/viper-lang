# Viper Language Build System
# Builds the runtime library and compiler

.PHONY: all clean runtime compiler test

CC = gcc
CFLAGS = -c -O3 -Wall -Wextra -fPIC
AR = ar
ARFLAGS = rcs

RUNTIME_SRC = runtime/runtime.c
RUNTIME_OBJ = runtime.o
RUNTIME_LIB = libviper.a

all: runtime compiler

# Build C runtime library
runtime: $(RUNTIME_SRC)
	$(CC) $(CFLAGS) $(RUNTIME_SRC) -o $(RUNTIME_OBJ)
	$(AR) $(ARFLAGS) $(RUNTIME_LIB) $(RUNTIME_OBJ)
	@echo "✅ Runtime library built: $(RUNTIME_LIB)"

# Build Rust compiler
compiler:
	cargo build --release
	@echo "✅ Compiler built: target/release/viper-lang"

# Clean build artifacts
clean:
	rm -f $(RUNTIME_OBJ) $(RUNTIME_LIB)
	cargo clean
	@echo "✅ Cleaned build artifacts"

# Run tests
test: compiler
	cargo test --release

# Install system-wide
install: compiler
	cp target/release/viper-lang /usr/local/bin/viper
	cp $(RUNTIME_LIB) /usr/local/lib/
	@echo "✅ Viper installed to /usr/local/bin/viper"
