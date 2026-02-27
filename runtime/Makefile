# Viper Runtime Makefile - Improved Version
# Builds the C runtime library for Viper language

# ============================================
# Configuration
# ============================================

CC = gcc
AR = ar

# Build profile (debug/release/profile)
PROFILE ?= release

# Version information
VERSION := $(shell git describe --tags 2>/dev/null || echo "0.4.1")

# Directories
SRC_DIR = src
OBJ_DIR = obj
LIB_DIR = lib
INCLUDE_DIR = include
INSTALL_PREFIX ?= /usr/local

# ============================================
# Build Profiles
# ============================================

ifeq ($(PROFILE),debug)
    CFLAGS = -Wall -Wextra -g -O0 -fPIC -std=c11 -D_GNU_SOURCE -DDEBUG
    LDFLAGS =
else ifeq ($(PROFILE),profile)
    CFLAGS = -Wall -Wextra -O3 -fPIC -std=c11 -D_GNU_SOURCE -pg -g
    LDFLAGS = -pg
else
    # Release (default)
    CFLAGS = -Wall -Wextra -O3 -fPIC -std=c11 -D_GNU_SOURCE -DNDEBUG
    LDFLAGS = -s
endif

# Common includes
COMMON_INCLUDES = -I$(INCLUDE_DIR) -I$(SRC_DIR) -I$(SRC_DIR)/concurrency

# Platform-specific flags
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    CFLAGS += -DPLATFORM_LINUX
    EVENT_LOOP_SRC = $(SRC_DIR)/event_loop_epoll.c
else ifeq ($(UNAME_S),Darwin)
    CFLAGS += -DPLATFORM_MACOS
    EVENT_LOOP_SRC = $(SRC_DIR)/event_loop_kqueue.c
else
    EVENT_LOOP_SRC = $(SRC_DIR)/event_loop_epoll.c
endif

# ============================================
# Source Files (Auto-discovered)
# ============================================

# Core sources
CORE_SRCS = $(SRC_DIR)/runtime.c

# Memory management
MEMORY_SRCS = $(wildcard $(SRC_DIR)/memory/*.c)

# Data structures
DS_SRCS = $(wildcard $(SRC_DIR)/data_structures/*.c)

# Concurrency
CONCURRENCY_SRCS = $(wildcard $(SRC_DIR)/concurrency/*.c)

# Fiber and scheduler
FIBER_SRCS = $(SRC_DIR)/fiber.c $(SRC_DIR)/fiber_pool.c $(SRC_DIR)/scheduler.c

# Event loop (platform-specific)
EVENT_LOOP_SRCS = $(EVENT_LOOP_SRC)

# All sources
ALL_SRCS = $(CORE_SRCS) $(MEMORY_SRCS) $(DS_SRCS) $(CONCURRENCY_SRCS) \
           $(FIBER_SRCS) $(EVENT_LOOP_SRCS)

# Object files (auto-generated from sources)
OBJS = $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(ALL_SRCS))

# Header files for dependency tracking
HEADERS = $(wildcard $(INCLUDE_DIR)/*.h) \
          $(wildcard $(SRC_DIR)/*.h) \
          $(wildcard $(SRC_DIR)/concurrency/*.h) \
          $(wildcard $(SRC_DIR)/memory/*.h) \
          $(wildcard $(SRC_DIR)/data_structures/*.h)

# Output library
LIBRARY = $(OBJ_DIR)/libviper.a

# ============================================
# Phony Targets
# ============================================

.PHONY: all clean install uninstall debug release profile dirs info help

# Default target
all: dirs $(LIBRARY)

# ============================================
# Build Rules
# ============================================

# Create output directory
dirs:
	@mkdir -p $(OBJ_DIR)

# Static library
$(LIBRARY): $(OBJS)
	@echo "Building static library: $@"
	@$(AR) rcs $@ $^
	@echo "Built: $@ ($(words $(OBJS)) objects)"

# Generic compilation rule with automatic dependency generation
$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c $(HEADERS)
	@mkdir -p $(dir $@)
	@echo "Compiling: $<"
	@$(CC) $(CFLAGS) $(COMMON_INCLUDES) -MMD -MP -c $< -o $@

# Include auto-generated dependencies
-include $(OBJS:.o=.d)

# ============================================
# Build Profiles
# ============================================

debug:
	@$(MAKE) PROFILE=debug all

release:
	@$(MAKE) PROFILE=release all

profile:
	@$(MAKE) PROFILE=profile all

# ============================================
# Installation
# ============================================

install: $(LIBRARY)
	@echo "Installing Viper runtime to $(INSTALL_PREFIX)"
	@mkdir -p $(INSTALL_PREFIX)/lib
	@mkdir -p $(INSTALL_PREFIX)/include/viper
	@cp $(LIBRARY) $(INSTALL_PREFIX)/lib/
	@cp $(INCLUDE_DIR)/*.h $(INSTALL_PREFIX)/include/viper/ 2>/dev/null || true
	@cp $(SRC_DIR)/*.h $(INSTALL_PREFIX)/include/viper/
	@cp $(SRC_DIR)/concurrency/*.h $(INSTALL_PREFIX)/include/viper/ 2>/dev/null || true
	@echo "Installation complete"

uninstall:
	@echo "Removing Viper runtime from $(INSTALL_PREFIX)"
	@rm -f $(INSTALL_PREFIX)/lib/libviper.a
	@rm -rf $(INSTALL_PREFIX)/include/viper
	@echo "Uninstallation complete"

# ============================================
# Maintenance
# ============================================

clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(OBJ_DIR) $(LIB_DIR)
	@echo "Clean complete"

distclean: clean
	@rm -f *~ $(SRC_DIR)/*~ $(SRC_DIR)/*/*~

# ============================================
# Information
# ============================================

info:
	@echo "Viper Runtime Build Information"
	@echo "================================"
	@echo "Version:     $(VERSION)"
	@echo "Profile:     $(PROFILE)"
	@echo "CC:          $(CC)"
	@echo "CFLAGS:      $(CFLAGS)"
	@echo "LDFLAGS:     $(LDFLAGS)"
	@echo "OBJ_DIR:     $(OBJ_DIR)"
	@echo "LIBRARY:     $(LIBRARY)"
	@echo "Source files: $(words $(ALL_SRCS))"
	@echo "Platform:    $(UNAME_S)"

help:
	@echo "Viper Runtime Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make              - Build release version"
	@echo "  make debug        - Build debug version"
	@echo "  make profile      - Build with profiling"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make install      - Install to system"
	@echo "  make uninstall    - Remove from system"
	@echo "  make info         - Show build information"
	@echo "  make help         - Show this help"
	@echo ""
	@echo "Options:"
	@echo "  PROFILE=debug|release|profile"
	@echo "  INSTALL_PREFIX=/path (default: /usr/local)"
