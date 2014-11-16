#include <string.h>

#include "miknet/mikdef.h"
#include "miknet/mikid.h"
#include "miknet/mikmeta.h"
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

static uint8_t *fragment_start(const mikpack_t *pack, uint16_t fragment)
{
	return pack->data + (fragment * MIKPACK_REAL_FRAG_SIZE);
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

int mikpack(mikpack_t *pack, const uint8_t *src, size_t len, uint8_t *dest)
{
	uint16_t frags;
	mikmeta_t metadata;
	size_t remainder;

	if (!pack || !src || !len || !dest)
		return MIKERR_BAD_PTR;

	pack->ref_count = 0;
	pack->data = dest;

	metadata.id = mikid();
	metadata.type = MIK_DATA;
	frags = fragments(len, &remainder);

	for (metadata.part = 0; metadata.part < frags; ++metadata.part) {
		if (metadata.part == frags - 1 && remainder)
			metadata.size = remainder;
		else
			metadata.size = MIKPACK_FRAG_SIZE;

		mikmeta_serialize(	&metadata,
					fragment_start(pack, metadata.part));
		memcpy(	mikpack_frag_data(pack, metadata.part),
			src + (metadata.part * MIKPACK_FRAG_SIZE),
			metadata.size);
	}

	return MIKERR_NONE;
}

int mikpack_frag(const mikpack_t *pack, uint16_t fragment, mikmeta_t *metadata)
{
	if (!pack || !metadata)
		return MIKERR_BAD_PTR;

	if (fragment > pack->frags - 1)
		return MIKERR_NO_SUCH_FRAG;;

	return mikmeta_deserialize(metadata, fragment_start(pack, fragment));
}

uint8_t *mikpack_frag_data(const mikpack_t *pack, uint16_t fragment)
{
	if (!pack)
		return NULL;

	if (fragment > pack->frags - 1)
		return NULL;

	return fragment_start(pack, fragment) + MIKMETA_SERIALIZED_OCTETS;
}
