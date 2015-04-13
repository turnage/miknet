#include <time.h>

#include "miknet/miktime.h"

static uint64_t miktime_spec_to_uint64(const struct timespec *tp)
{
	return (tp->tv_sec * 1000000000) + tp->tv_nsec;
}

uint64_t miktime()
{
	struct timespec tp;

	clock_gettime(CLOCK_MONOTONIC, &tp);

	return miktime_spec_to_uint64(&tp);
}

uint64_t miktime_sleep(uint64_t nsecs)
{
	struct timespec tp;
	struct timespec retry;
	uint64_t retry_nsecs;

	tp.tv_sec = nsecs / 1000000000;
	tp.tv_nsec = nsecs % 1000000000;

	clock_nanosleep(CLOCK_MONOTONIC, 0, &tp, &retry);
	retry_nsecs = miktime_spec_to_uint64(&retry);

	return miktime_spec_to_uint64(&retry);
}
