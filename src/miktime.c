#include <time.h>

#include "miknet/miktime.h"

uint64_t miktime()
{
	struct timespec tp;

	clock_gettime(CLOCK_MONOTONIC, &tp);

	return (tp.tv_sec * 1000000000) + tp.tv_nsec;
}
