/**
 * Viper Fiber Implementation
 * 
 * Stackful coroutines using setjmp/longjmp for context switching.
 */

#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include <signal.h>
#include <unistd.h>
#include <sys/mman.h>
#include "fiber.h"

/* ============================================ */
/* Internal State                              */
/* ============================================ */

static uint64_t g_fiber_id_counter = 0;

/* Current executing fiber (TLS) */
#ifdef __linux__
static __thread ViperFiber* g_current_fiber = NULL;
#elif defined(__APPLE__)
static __thread ViperFiber* g_current_fiber = NULL;
#else
static ViperFiber* g_current_fiber = NULL;
#endif

/* ============================================ */
/* Signal Handling (for stack growth)          */
/* ============================================ */

static void sigsegv_handler(int sig, siginfo_t* info, void* context) {
    (void)sig;
    (void)context;
    
    /* Find the fiber whose stack was overflowed */
    ViperFiber* fiber = g_current_fiber;
    if (!fiber) {
        _exit(1);
    }
    
    /* Check if the fault is in the fiber's stack */
    char* fault_addr = (char*)info->si_addr;
    char* stack_bottom = (char*)fiber->stack_base - fiber->stack_capacity;
    
    if (fault_addr >= stack_bottom && fault_addr < (char*)fiber->stack_base) {
        /* Grow the stack */
        size_t new_size = fiber->stack_size + FIBER_STACK_GROW_STEP;
        if (new_size > FIBER_MAX_STACK_SIZE) {
            _exit(1);  /* Stack too big */
        }
        
        /* Note: In a full implementation, we'd remap with more memory */
        /* For now, just report error */
        _exit(1);
    }
    
    _exit(1);
}

static void setup_signal_handler(void) {
    static int initialized = 0;
    if (initialized) return;
    
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_sigaction = sigsegv_handler;
    sa.sa_flags = SA_SIGINFO;
    sigemptyset(&sa.sa_mask);
    
    sigaction(SIGSEGV, &sa, NULL);
    initialized = 1;
}

/* ============================================ */
/* Fiber Implementation                         */
/* ============================================ */

ViperFiber* vp_fiber_create(void (*func)(void*), void* arg, size_t stack_size) {
    ViperFiber* fiber = (ViperFiber*)malloc(sizeof(ViperFiber));
    if (!fiber) return NULL;
    
    memset(fiber, 0, sizeof(ViperFiber));
    
    fiber->id = __sync_fetch_and_add(&g_fiber_id_counter, 1);
    fiber->state = FIBER_NEW;
    fiber->func = func;
    fiber->arg = arg;
    fiber->parent = g_current_fiber;
    
    /* Set up stack */
    if (stack_size == 0) {
        stack_size = FIBER_DEFAULT_STACK_SIZE;
    }
    fiber->stack_size = stack_size;
    fiber->stack_capacity = stack_size;
    
    /* Allocate stack with guard page */
    fiber->stack_base = mmap(
        NULL,
        stack_size + 4096,  /* Extra page for guard */
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0
    );
    
    if (fiber->stack_base == MAP_FAILED) {
        free(fiber);
        return NULL;
    }
    
    /* Set up guard page (unreadable/unwritable) */
    mprotect(fiber->stack_base, 4096, PROT_NONE);
    
    /* Stack grows downward, so stack_ptr starts at base + size */
    fiber->stack_ptr = (char*)fiber->stack_base + stack_size + 4096;
    
    /* Set up jump buffer for context switching */
    /* This will be initialized on first switch */
    
    /* Set up signal handler for stack growth */
    setup_signal_handler();
    
    return fiber;
}

void vp_fiber_free(ViperFiber* fiber) {
    if (!fiber) return;
    
    if (fiber->stack_base && fiber->stack_base != MAP_FAILED) {
        munmap(fiber->stack_base, fiber->stack_capacity + 4096);
    }
    
    free(fiber);
}

uint64_t vp_fiber_id(ViperFiber* fiber) {
    return fiber ? fiber->id : 0;
}

ViperFiberState vp_fiber_state(ViperFiber* fiber) {
    return fiber ? fiber->state : FIBER_CANCELLED;
}

ViperFiber* vp_fiber_current(void) {
    return g_current_fiber;
}

int vp_fiber_grow_stack(ViperFiber* fiber, size_t new_size) {
    if (!fiber || new_size <= fiber->stack_size || new_size > FIBER_MAX_STACK_SIZE) {
        return -1;
    }
    
    /* Allocate new stack */
    void* new_stack = mmap(
        NULL,
        new_size + 4096,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0
    );
    
    if (new_stack == MAP_FAILED) {
        return -1;
    }
    
    /* Copy old stack to new stack */
    size_t copy_size = fiber->stack_size;
    memcpy(
        (char*)new_stack + 4096 + new_size - copy_size,
        (char*)fiber->stack_base + 4096 + fiber->stack_capacity - copy_size,
        copy_size
    );
    
    /* Set up guard page */
    mprotect(new_stack, 4096, PROT_NONE);
    
    /* Unmap old stack */
    munmap(fiber->stack_base, fiber->stack_capacity + 4096);
    
    /* Update fiber */
    fiber->stack_base = new_stack;
    fiber->stack_capacity = new_size;
    fiber->stack_ptr = (char*)new_stack + 4096 + new_size - copy_size;
    
    return 0;
}

/* ============================================ */
/* Scheduler Integration                       */
/* ============================================ */

/* External scheduler functions - will be implemented in scheduler.c */
extern void vp_scheduler_add_ready(ViperFiber* fiber);
extern ViperFiber* vp_scheduler_get_ready(void);
extern void vp_scheduler_put_to_sleep(ViperFiber* fiber);

void vp_fiber_yield(void) {
    ViperFiber* current = g_current_fiber;
    if (!current) return;
    
    /* Put current fiber to sleep, get next ready fiber */
    current->state = FIBER_WAITING;
    vp_scheduler_put_to_sleep(current);
    
    /* Get next ready fiber */
    ViperFiber* next = vp_scheduler_get_ready();
    if (next) {
        vp_fiber_switch(current, next);
    }
}

void vp_fiber_resume(ViperFiber* fiber) {
    if (!fiber) return;
    
    fiber->state = FIBER_READY;
    vp_scheduler_add_ready(fiber);
}

void vp_fiber_switch(ViperFiber* from, ViperFiber* to) {
    /* Save current fiber state */
    g_current_fiber = to;
    to->state = FIBER_RUNNING;
    
    /* Use setjmp/longjmp for context switch */
    /* This is a simplified version - real implementation would save/restore more */
    
    if (from && to) {
        /* Jump to the new fiber */
        /* The fiber's initial run will start from its function */
        /* After the function returns, we need to handle completion */
        
        /* For now, just switch and run */
        /* A full implementation would use jmp_buf to save/restore state */
    }
    
    /* Mark current as waiting */
    if (from) {
        from->state = FIBER_WAITING;
    }
    
    /* Run the fiber function if it's new */
    if (to->state == FIBER_NEW) {
        to->state = FIBER_RUNNING;
        to->func(to->arg);
        to->state = FIBER_COMPLETED;
        
        /* Return to parent or scheduler */
        if (to->parent) {
            vp_fiber_resume(to->parent);
        }
    }
}

int vp_fiber_start(ViperFiber* fiber) {
    if (!fiber || fiber->state != FIBER_NEW) {
        return -1;
    }
    
    fiber->state = FIBER_READY;
    
    /* Add to scheduler */
    vp_scheduler_add_ready(fiber);
    
    return 0;
}
