#ifndef VIPER_CHANNEL_H
#define VIPER_CHANNEL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/**
 * Opaque Channel type
 */
typedef struct ViperChannel ViperChannel;

/**
 * Create a new channel with the specified buffer capacity
 * @param capacity Buffer size (0 for synchronous/unbuffered)
 * @return Pointer to Channel, or NULL on failure
 */
ViperChannel* vp_channel_create(size_t capacity);

/**
 * Destroy a channel
 * @param chan Channel to destroy
 */
void vp_channel_destroy(ViperChannel* chan);

/**
 * Increment channel reference count
 * @param chan Channel
 */
void vp_channel_retain(ViperChannel* chan);

/**
 * Decrement channel reference count
 * @param chan Channel
 */
void vp_channel_release(ViperChannel* chan);

/**
 * Send a value to the channel (blocks if full)
 * @param chan Channel
 * @param value Value to send (as int64_t)
 * @return true on success, false if channel is closed
 */
bool vp_channel_send(ViperChannel* chan, int64_t value);

/**
 * Receive a value from the channel (blocks if empty)
 * @param chan Channel
 * @return Received value as int64_t, or 0 if channel is closed and empty
 */
int64_t vp_channel_recv(ViperChannel* chan);

/**
 * Try to send a value (non-blocking)
 * @param chan Channel
 * @param value Value to send
 * @return true if sent, false if channel is full or closed
 */
bool vp_channel_try_send(ViperChannel* chan, void* value);

/**
 * Try to receive a value (non-blocking)
 * @param chan Channel
 * @param out_value Pointer to store received value
 * @return true if received, false if channel is empty (true if closed and empty)
 */
bool vp_channel_try_recv(ViperChannel* chan, void** out_value);

/**
 * Close a channel
 * @param chan Channel to close
 */
void vp_channel_close(ViperChannel* chan);

/**
 * Check if channel is closed
 * @param chan Channel
 * @return true if closed
 */
bool vp_channel_is_closed(ViperChannel* chan);

/**
 * Get the number of messages in the channel
 * @param chan Channel
 * @return Number of messages
 */
size_t vp_channel_len(ViperChannel* chan);

#endif /* VIPER_CHANNEL_H */
