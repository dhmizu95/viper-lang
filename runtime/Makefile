# Viper Runtime Makefile
# Builds the C runtime library for Viper language

CC = gcc
CFLAGS = -Wall -Wextra -O2 -fPIC -std=c11
INCLUDE = -I./include

# Source files
SRC_DIR = src
MEMORY_SRC = $(SRC_DIR)/memory/arc.c
DS_SRC = $(SRC_DIR)/data_structures/list.c
RUNTIME_SRC = $(SRC_DIR)/runtime.c

# Object files
OBJ_DIR = obj
MEMORY_OBJ = $(OBJ_DIR)/arc.o
DS_OBJ = $(OBJ_DIR)/list.o
RUNTIME_OBJ = $(OBJ_DIR)/runtime.o

# Output
LIB_NAME = libviper.a
LIB_PATH = $(LIB_NAME)

# All source files
ALL_SRC = $(RUNTIME_SRC) $(MEMORY_SRC) $(DS_SRC)

.PHONY: all clean dirs

all: dirs $(LIB_PATH)

dirs:
	@mkdir -p $(OBJ_DIR)

$(LIB_PATH): $(RUNTIME_OBJ) $(MEMORY_OBJ) $(DS_OBJ)
	ar rcs $@ $^
	@echo "Built static library: $(LIB_PATH)"

$(OBJ_DIR)/runtime.o: $(RUNTIME_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/arc.o: $(MEMORY_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/list.o: $(DS_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

clean:
	rm -rf $(OBJ_DIR) $(LIB_PATH)
	@echo "Cleaned runtime library"

# Install to system library path (optional)
install: $(LIB_PATH)
	cp $(LIB_PATH) /usr/local/lib/
	cp -r include/* /usr/local/include/
	@echo "Installed Viper runtime to system"

# Development install (local)
install-local: $(LIB_PATH)
	cp $(LIB_PATH) ../
	@echo "Installed Viper runtime to project root"
