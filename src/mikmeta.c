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

int mikmeta_serialize(const mikmeta_t *metadata, uint8_t *destination)
{
	if (!metadata || !destination) {
		return -1;
	}

	destination[0] = get_mso(metadata->id);
	destination[1] = get_lso(metadata->id);
	destination[2] = get_mso(metadata->part);
	destination[3] = get_lso(metadata->part);
	destination[4] = metadata->type;
	destination[5] = metadata->channel;
	destination[6] = get_mso(metadata->size);
	destination[7] = get_lso(metadata->size);

	return 0;
}

int mikmeta_deserialize(mikmeta_t *metadata, const uint8_t *serialized)
{
	if (!metadata || !serialized)
		return MIKERR_BAD_PTR;

	metadata->id = combine(serialized[0], serialized[1]);
	metadata->part = combine(serialized[2], serialized[3]);
	metadata->type = serialized[4];
	metadata->channel = serialized[5];
	metadata->size = combine(serialized[6], serialized[7]);

	return MIKERR_NONE;;
}
