#include "miknet/mikpack.h"

/**
 *  Returns how many fragments a packet of len data should be broken into.
 *  rounding up. E.g. for 9 bytes and 2 byte fragments, make 5 fragments.
 */
static uint16_t fragments(size_t len, size_t *remainder)
{
	uint16_t frags = len / MIKPACK_FRAG_SIZE;

	if ((*remainder = len % MIKPACK_FRAG_SIZE) != 0)
		++frags;

	return frags;
}

/**
 *  Returns the amount of octets required to store the given amount of
 *  fragments.
 */
static size_t fragment_data_size(uint16_t frags)
{
	return frags * MIKPACK_FRAG_SIZE;
}

/**
 *  Returns the amount of octets required to store a semi fragment with len
 *  octets.
 */
static size_t semi_fragment_data_size(size_t len)
{
	return len + MIKFRAG_HEADER_SIZE;
}

size_t mikpack_mem_est(size_t len)
{
	size_t remainder;
	size_t mem_est = fragment_data_size(fragments(len, &remainder));

	if (remainder != 0 || len == 0)
		mem_est += semi_fragment_data_size(remainder);

	return mem_est;
}
