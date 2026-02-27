# Viper Runtime Makefile
# Builds the C runtime library for Viper language

CC = gcc
CFLAGS = -Wall -Wextra -O3 -fPIC -std=c11 -D_GNU_SOURCE
INCLUDE = -I./include

# Source files
SRC_DIR = src
RUNTIME_SRC = $(SRC_DIR)/runtime.c
MEMORY_SRC = $(SRC_DIR)/memory/arc.c
MEMORY_POOL_SRC = $(SRC_DIR)/memory/pool.c
MEMORY_ALLOCATOR_SRC = $(SRC_DIR)/memory/allocator.c
DS_SRC = $(SRC_DIR)/data_structures/list.c
LIST_BOOL_SRC = $(SRC_DIR)/data_structures/list_bool.c
BITVEC_SRC = $(SRC_DIR)/data_structures/bitvec.c
DICT_SRC = $(SRC_DIR)/data_structures/dict.c
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

# Object files
OBJ_DIR = obj

.PHONY: all clean dirs

all: dirs $(OBJ_DIR)/libviper.a

dirs:
	@mkdir -p $(OBJ_DIR)

$(OBJ_DIR)/libviper.a: $(OBJ_DIR)/runtime.o $(OBJ_DIR)/arc.o $(OBJ_DIR)/pool.o $(OBJ_DIR)/allocator.o $(OBJ_DIR)/list.o $(OBJ_DIR)/list_bool.o $(OBJ_DIR)/bitvec.o $(OBJ_DIR)/dict.o $(OBJ_DIR)/bigint.o $(OBJ_DIR)/concurrency.o $(OBJ_DIR)/channel.o $(OBJ_DIR)/wait_group.o $(OBJ_DIR)/task_queue.o $(OBJ_DIR)/thread_pool.o $(OBJ_DIR)/semaphore.o $(OBJ_DIR)/rwlock.o $(OBJ_DIR)/barrier.o $(OBJ_DIR)/atomic.o $(OBJ_DIR)/async.o $(OBJ_DIR)/fiber.o $(OBJ_DIR)/scheduler.o $(OBJ_DIR)/event_loop_epoll.o
	ar rcs $@ $^
	@echo "Built static library: $(OBJ_DIR)/libviper.a"

$(OBJ_DIR)/runtime.o: $(RUNTIME_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/arc.o: $(MEMORY_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/pool.o: $(MEMORY_POOL_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/allocator.o: $(MEMORY_ALLOCATOR_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/list.o: $(DS_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/list_bool.o: $(LIST_BOOL_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/bitvec.o: $(BITVEC_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/dict.o: $(DICT_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/bigint.o: $(BIGINT_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/concurrency.o: $(CONCURRENCY_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/channel.o: $(CHANNEL_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/wait_group.o: $(WAITGROUP_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/task_queue.o: $(TASKQUEUE_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/thread_pool.o: $(THREADPOOL_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/semaphore.o: $(SEMAPHORE_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/rwlock.o: $(RWLOCK_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/barrier.o: $(BARRIER_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -pthread -c $< -o $@

$(OBJ_DIR)/atomic.o: $(ATOMIC_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -I$(SRC_DIR)/concurrency -c $< -o $@

$(OBJ_DIR)/async.o: $(ASYNC_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

$(OBJ_DIR)/fiber.o: $(FIBER_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -pthread -c $< -o $@

$(OBJ_DIR)/scheduler.o: $(SCHEDULER_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -pthread -c $< -o $@

$(OBJ_DIR)/event_loop_epoll.o: $(EVENT_LOOP_SRC)
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

clean:
	rm -rf $(OBJ_DIR)
	@echo "Cleaned runtime library"
