#include "miknet/mikmeta.h"

int mikmeta_serialize(const mikmeta_t *metadata, uint8_t *destination)
{
	if (!metadata || !destination) {
		return -1;
	}

	destination[0] = metadata->type;
	destination[1] = metadata->size >> 8;
	destination[2] = metadata->size & 0xff;

	return 0;
}

mikmeta_t mikmeta_deserialize(const uint8_t *serialized)
{
	mikmeta_t metadata;

	if (!serialized) {
		metadata.type = MIK_NONE;
		return metadata;
	}

	metadata.type = serialized[0];

	metadata.size = 0;
	metadata.size ^= serialized[1] << 8;
	metadata.size ^= serialized[2];

	return metadata;
}
