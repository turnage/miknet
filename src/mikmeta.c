#include "miknet/mikdef.h"
#include "miknet/mikmeta.h"

/**
 *  Fetches the most significant octet in a 16 bit integer.
 */
static uint8_t get_mso(const uint16_t octets) { return octets >> 8; }

/**
 *  Fetches the least significant octet in a 16 bit integer.
 */
static uint8_t get_lso(const uint16_t octets) { return octets & 0xff; }

static uint16_t combine(const uint8_t mso, const uint8_t lso)
{
	uint16_t combination = 0;

	combination ^= mso << 8;
	combination ^= lso;
	return combination;
}

int mikmeta_serialize(const mikmeta_t *meta, uint8_t *destination)
{
	if (!meta || !destination) {
		return -1;
	}

	destination[0] = get_mso(meta->id);
	destination[1] = get_lso(meta->id);
	destination[2] = get_mso(meta->part);
	destination[3] = get_lso(meta->part);
	destination[4] = meta->type;
	destination[5] = meta->channel;
	destination[6] = get_mso(meta->size);
	destination[7] = get_lso(meta->size);

	return 0;
}

int mikmeta_deserialize(mikmeta_t *meta, const uint8_t *serialized)
{
	if (!meta || !serialized)
		return MIKERR_BAD_PTR;

	meta->id = combine(serialized[0], serialized[1]);
	meta->part = combine(serialized[2], serialized[3]);
	meta->type = serialized[4];
	meta->channel = serialized[5];
	meta->size = combine(serialized[6], serialized[7]);

	return MIKERR_NONE;;
}
