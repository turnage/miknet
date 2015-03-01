#include <stdlib.h>
#include <string.h>

#include "miknet/mikdef.h"
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

/**
 *  Returns a pointer to the start of a fragment's data.
 */
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

/**
 *  Estimates the memory required to hold a packet's data.
 */
static size_t mikpack_mem_est(size_t len)
{
	size_t remainder;
	size_t mem_est = fragment_data_size(fragments(len, &remainder));

	if (remainder != 0 || len == 0)
		mem_est += semi_fragment_data_size(remainder);

	return mem_est;
}

int mikpack(	mikpack_t **pack,
		miktype_t type,
		const uint8_t *src,
		size_t len)
{
	mikmeta_t meta;
	size_t remainder;

	if (!pack || !src || !len)
		return MIKERR_BAD_PTR;

	*pack = malloc(mikpack_mem_est(len) + sizeof(mikpack_t));

	if (!(*pack))
		return MIKERR_BAD_MEM;

	(*pack)->frags = fragments(len, &remainder);
	(*pack)->ref_count = 0;
	(*pack)->data = (uint8_t *)*pack + sizeof(mikpack_t);

	meta.type = type;

	for (meta.part = 0; meta.part < (*pack)->frags; ++meta.part) {
		if (meta.part == (*pack)->frags - 1 && remainder)
			meta.size = remainder;
		else
			meta.size = MIKPACK_FRAG_SIZE;

		mikmeta_serialize(	&meta,
					fragment_start((*pack), meta.part));
		memcpy(	mikpack_frag_data((*pack), meta.part),
			src + (meta.part * MIKPACK_FRAG_SIZE),
			meta.size);
	}

	return MIKERR_NONE;
}

int mikpack_set_id(mikpack_t *pack, uint16_t id)
{
	int i;

	if (!pack)
		return MIKERR_BAD_PTR;

	for (i = 0; i < pack->frags; ++i) {
		uint8_t *meta_start = mikpack_frag_raw_data(pack, i);
		mikmeta_t meta = *((mikmeta_t *)meta_start);
		meta.id = id;
		mikmeta_serialize(&meta, meta_start);
	}

	return MIKERR_NONE;
}

int mikpack_frag(const mikpack_t *pack, uint16_t fragment, mikmeta_t *meta)
{
	if (!pack || !meta)
		return MIKERR_BAD_PTR;

	if (fragment > pack->frags - 1)
		return MIKERR_NO_SUCH_FRAG;;

	return mikmeta_deserialize(meta, fragment_start(pack, fragment));
}

uint8_t *mikpack_frag_data(const mikpack_t *pack, uint16_t frag)
{
	uint8_t *frag_start = mikpack_frag_raw_data(pack, frag);

	return frag_start ? frag_start + MIKMETA_OCTETS : NULL;
}

uint8_t *mikpack_frag_raw_data(const mikpack_t *pack, uint16_t frag)
{
	if (!pack)
		return NULL;

	if (frag > pack->frags - 1)
		return NULL;

	return fragment_start(pack, frag);
}

void mikpack_close(mikpack_t *pack)
{
	free(pack);
}
