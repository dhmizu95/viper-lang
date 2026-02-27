/**
 * Simple test for lock-free channel
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include <time.h>
#include "runtime/src/concurrency/channel.h"
#include "runtime/src/scheduler.h"

static _Atomic int64_t sent_count = 0;
static _Atomic int64_t recv_count = 0;

void producer_task(void* arg) {
    ViperChannel* chan = (ViperChannel*)arg;
    for (int i = 0; i < 1000; i++) {
        vp_channel_send(chan, i);
    }
    atomic_fetch_add(&sent_count, 1000);
}

void consumer_task(void* arg) {
    ViperChannel* chan = (ViperChannel*)arg;
    int64_t sum = 0;
    for (int i = 0; i < 1000; i++) {
        sum += vp_channel_recv(chan);
    }
    atomic_fetch_add(&recv_count, 1000);
    printf("Consumer done, sum=%ld\n", sum);
}

int main() {
    printf("Lock-free channel test\n");
    
    ViperChannel* chan = vp_channel_create(100);
    if (!chan) {
        printf("Failed to create channel\n");
        return 1;
    }
    printf("Channel created (capacity=100)\n");
    
    atomic_store(&sent_count, 0);
    atomic_store(&recv_count, 0);
    
    double start = clock();
    
    vp_scheduler_init(0);
    printf("Scheduler initialized\n");
    
    /* Spawn 10 producer-consumer pairs */
    for (int i = 0; i < 10; i++) {
        vp_scheduler_submit_task(producer_task, chan);
        vp_scheduler_submit_task(consumer_task, chan);
    }
    printf("Tasks submitted\n");
    
    vp_scheduler_wait_all();
    printf("All tasks completed\n");
    
    vp_scheduler_shutdown();
    
    double elapsed = (clock() - start) / CLOCKS_PER_SEC * 1000;
    
    printf("Sent: %ld\n", atomic_load(&sent_count));
    printf("Recv: %ld\n", atomic_load(&recv_count));
    printf("Time: %.2f ms\n", elapsed);
    
    vp_channel_destroy(chan);
    
    return 0;
}
