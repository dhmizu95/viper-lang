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
RUNTIME_SRC = $(SRC_DIR)/runtime.c
MEMORY_SRC = $(SRC_DIR)/memory/arc.c
MEMORY_POOL_SRC = $(SRC_DIR)/memory/pool.c
MEMORY_ALLOCATOR_SRC = $(SRC_DIR)/memory/allocator.c
DS_SRC = $(SRC_DIR)/data_structures/list.c
LIST_BOOL_SRC = $(SRC_DIR)/data_structures/list_bool.c
BITVEC_SRC = $(SRC_DIR)/data_structures/bitvec.c
DICT_SRC = $(SRC_DIR)/data_structures/dict.c
BYTES_SRC = $(SRC_DIR)/data_structures/bytes.c
BIGINT_SRC = $(SRC_DIR)/bigint.c
CONCURRENCY_SRC = $(SRC_DIR)/concurrency/concurrency.c
CHANNEL_SRC = $(SRC_DIR)/concurrency/channel.c
WAITGROUP_SRC = $(SRC_DIR)/concurrency/wait_group.c
TASKQUEUE_SRC = $(SRC_DIR)/concurrency/task_queue.c
THREADPOOL_SRC = $(SRC_DIR)/concurrency/thread_pool.c
SEMAPHORE_SRC = $(SRC_DIR)/concurrency/semaphore.c
RWLOCK_SRC = $(SRC_DIR)/concurrency/rwlock.c
BARRIER_SRC = $(SRC_DIR)/concurrency/barrier.c
ATOMIC_SRC = $(SRC_DIR)/concurrency/atomic.c
ASYNC_SRC = $(SRC_DIR)/async.c
FIBER_SRC = $(SRC_DIR)/fiber.c
SCHEDULER_SRC = $(SRC_DIR)/scheduler.c
EVENT_LOOP_SRC = $(SRC_DIR)/event_loop_epoll.c

# Standard library modules (Phase 1-4)
SYS_SRC = $(SRC_DIR)/sys.c
OS_SRC = $(SRC_DIR)/os.c
TIME_MOD_SRC = $(SRC_DIR)/time_mod.c
MATH_MOD_SRC = $(SRC_DIR)/math_mod.c
JSON_SRC = $(SRC_DIR)/json.c
COLLECTIONS_SRC = $(SRC_DIR)/collections.c
RE_MOD_SRC = $(SRC_DIR)/re_mod.c
RANDOM_MOD_SRC = $(SRC_DIR)/random_mod.c
SOCKET_MOD_SRC = $(SRC_DIR)/socket_mod.c
ASYNCIO_MOD_SRC = $(SRC_DIR)/asyncio_mod.c
HTTP_MOD_SRC = $(SRC_DIR)/http_mod.c
SELECT_MOD_SRC = $(SRC_DIR)/select_mod.c
HASHLIB_SRC = $(SRC_DIR)/hashlib.c
DECIMAL_MOD_SRC = $(SRC_DIR)/decimal_mod.c
LOGGING_SRC = $(SRC_DIR)/logging.c
GC_SRC = $(SRC_DIR)/gc.c

# Object files
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
    # Release (default) - with LTO for better optimization
    CFLAGS = -Wall -Wextra -O3 -fPIC -std=c11 -D_GNU_SOURCE -DNDEBUG -flto -ffat-lto-objects -finline-functions -finline-small-functions
    LDFLAGS = -s -flto
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

# Async runtime
ASYNC_RUNTIME_SRC = $(SRC_DIR)/async_runtime.c

# Event loop (platform-specific)
EVENT_LOOP_SRCS = $(EVENT_LOOP_SRC)

# All sources
ALL_SRCS = $(CORE_SRCS) $(MEMORY_SRCS) $(DS_SRCS) $(CONCURRENCY_SRCS) \
           $(FIBER_SRCS) $(ASYNC_RUNTIME_SRC) $(EVENT_LOOP_SRCS)

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

$(OBJ_DIR)/libviper.a: $(OBJ_DIR)/runtime.o $(OBJ_DIR)/arc.o $(OBJ_DIR)/pool.o $(OBJ_DIR)/allocator.o $(OBJ_DIR)/list.o $(OBJ_DIR)/list_bool.o $(OBJ_DIR)/bitvec.o $(OBJ_DIR)/dict.o $(OBJ_DIR)/bytes.o $(OBJ_DIR)/bigint.o $(OBJ_DIR)/concurrency.o $(OBJ_DIR)/channel.o $(OBJ_DIR)/wait_group.o $(OBJ_DIR)/task_queue.o $(OBJ_DIR)/thread_pool.o $(OBJ_DIR)/semaphore.o $(OBJ_DIR)/rwlock.o $(OBJ_DIR)/barrier.o $(OBJ_DIR)/atomic.o $(OBJ_DIR)/async.o $(OBJ_DIR)/fiber.o $(OBJ_DIR)/scheduler.o $(OBJ_DIR)/event_loop_epoll.o $(OBJ_DIR)/sys.o $(OBJ_DIR)/os.o $(OBJ_DIR)/time_mod.o $(OBJ_DIR)/gc.o $(OBJ_DIR)/math_mod.o $(OBJ_DIR)/json.o $(OBJ_DIR)/collections.o $(OBJ_DIR)/re_mod.o $(OBJ_DIR)/random_mod.o $(OBJ_DIR)/socket_mod.o $(OBJ_DIR)/asyncio_mod.o $(OBJ_DIR)/http_mod.o $(OBJ_DIR)/select_mod.o $(OBJ_DIR)/hashlib.o $(OBJ_DIR)/decimal_mod.o $(OBJ_DIR)/logging.o
	ar rcs $@ $^
	@echo "Built static library: $(OBJ_DIR)/libviper.a"

$(OBJ_DIR)/runtime.o: $(RUNTIME_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/arc.o: $(MEMORY_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/pool.o: $(MEMORY_POOL_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/allocator.o: $(MEMORY_ALLOCATOR_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/list.o: $(DS_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/list_bool.o: $(LIST_BOOL_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/bitvec.o: $(BITVEC_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/dict.o: $(DICT_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/bytes.o: $(BYTES_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/bigint.o: $(BIGINT_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/concurrency.o: $(CONCURRENCY_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/channel.o: $(CHANNEL_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/wait_group.o: $(WAITGROUP_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/task_queue.o: $(TASKQUEUE_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/thread_pool.o: $(THREADPOOL_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/semaphore.o: $(SEMAPHORE_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/rwlock.o: $(RWLOCK_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/barrier.o: $(BARRIER_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/atomic.o: $(ATOMIC_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/async.o: $(ASYNC_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/fiber.o: $(FIBER_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -pthread -c $< -o $@

$(OBJ_DIR)/scheduler.o: $(SCHEDULER_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -pthread -c $< -o $@

$(OBJ_DIR)/event_loop_epoll.o: $(EVENT_LOOP_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

# ============================================
# Standard Library Modules (Phase 1)
# ============================================

$(OBJ_DIR)/sys.o: $(SYS_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/os.o: $(OS_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/time_mod.o: $(TIME_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -lm -c $< -o $@

$(OBJ_DIR)/gc.o: $(GC_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

# ============================================
# Standard Library Modules (Phase 2)
# ============================================

$(OBJ_DIR)/math_mod.o: $(MATH_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -lm -c $< -o $@

$(OBJ_DIR)/json.o: $(JSON_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/collections.o: $(COLLECTIONS_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/re_mod.o: $(RE_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/random_mod.o: $(RANDOM_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

# ============================================
# Standard Library Modules (Phase 3)
# ============================================

$(OBJ_DIR)/socket_mod.o: $(SOCKET_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/asyncio_mod.o: $(ASYNCIO_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/http_mod.o: $(HTTP_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/select_mod.o: $(SELECT_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

# ============================================
# Standard Library Modules (Phase 4)
# ============================================

$(OBJ_DIR)/hashlib.o: $(HASHLIB_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/decimal_mod.o: $(DECIMAL_MOD_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -c $< -o $@

$(OBJ_DIR)/logging.o: $(LOGGING_SRC)
	$(CC) $(CFLAGS) $(COMMON_INCLUDES) -pthread -c $< -o $@

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
