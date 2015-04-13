#ifndef MIKNET_MIKTIME_H_
#define MIKNET_MIKTIME_H_

#include <stdint.h>

/**
 *  Returns the current time in nanoseconds. Origin time arbitrary.
 */
uint64_t miktime();

/**
 *  Sleeps the calling thread for the specified time in nanoseconds. Returns the
 *  remaining unslept time in nanoseconds.
 */
uint64_t miktime_sleep(uint64_t nsecs);

#endif /* MIKNET_MIKTIME_H_ */
