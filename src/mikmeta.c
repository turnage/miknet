#include "miknet/mikmeta.h"

/**
 *  Fetches the most significant octet in a 16 bit integer.
 */
static uint8_t get_mso(const uint16_t octets) { return octets >> 8; }

/**
 *  Fetches the least significant octet in a 16 bit integer.
 */
static uint8_t get_lso(const uint16_t octets) { return octets >> 8; }

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

	destination[0] = metadata->id >> 8;
	destination[1] = metadata->id & 0xff;
	destination[2] = metadata->part >> 8;
	destination[3] = metadata->part & 0xff;
	destination[4] = metadata->type;
	destination[5] = metadata->size >> 8;
	destination[6] = metadata->size & 0xff;

	return 0;
}

mikmeta_t mikmeta_deserialize(const uint8_t *serialized)
{
	mikmeta_t metadata;

	if (!serialized) {
		metadata.type = MIK_NONE;
		return metadata;
	}

	metadata.id = combine(serialized[0], serialized[1]);
	metadata.part = combine(serialized[2], serialized[3]);
	metadata.type = serialized[4];
	metadata.size = combine(serialized[5], serialized[6]);

	return metadata;
}
