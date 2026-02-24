#ifndef VIPER_WAIT_GROUP_H
#define VIPER_WAIT_GROUP_H

#include <stdint.h>

/**
 * Opaque WaitGroup type
 */
typedef struct ViperWaitGroup ViperWaitGroup;

/**
 * Create a new WaitGroup
 * @return Pointer to WaitGroup, or NULL on failure
 */
ViperWaitGroup* vp_waitgroup_create_impl(void);

/**
 * Destroy a WaitGroup
 * @param wg WaitGroup to destroy
 */
void vp_waitgroup_destroy_impl(ViperWaitGroup* wg);

/**
 * Add delta to the WaitGroup counter
 * @param wg WaitGroup
 * @param delta Value to add (typically positive)
 */
void vp_waitgroup_add_impl(ViperWaitGroup* wg, int64_t delta);

/**
 * Decrement the WaitGroup counter by 1
 * @param wg WaitGroup
 */
void vp_waitgroup_done_impl(ViperWaitGroup* wg);

/**
 * Wait until the counter reaches zero
 * @param wg WaitGroup
 */
void vp_waitgroup_wait_impl(ViperWaitGroup* wg);

/**
 * Get the current counter value
 * @param wg WaitGroup
 * @return Current counter value
 */
int64_t vp_waitgroup_count(ViperWaitGroup* wg);

#endif /* VIPER_WAIT_GROUP_H */
