#ifndef MIKNET_MIKMETA_H_
#define MIKNET_MIKMETA_H_

#include <stdint.h>
#include <stddef.h>

#define MIKMETA_SERIALIZED_OCTETS 3

typedef enum {
	MIK_NONE = 0,
	MIK_JOIN = 1,
	MIK_QUIT = 2,
	MIK_DATA = 3
} mikflag_t;

typedef struct mikmeta_t {
	mikflag_t type;
	uint16_t size;
} mikmeta_t;

/**
 *  Serializes the information in a Miknet metadata object to be sent over
 *  the network.
 *
 *  Returns 0 on success.
 */
int mikmeta_serialize(const mikmeta_t *metadata, uint8_t *destination);

/**
 *  Deserializes data that represents Miknet metadata. The data should be
 *  MIKMETA_SERIALIZED_OCTETS long.
 *
 *  Returns a constructed metadata object.
 */
mikmeta_t mikmeta_deserialize(const uint8_t *serialized);

#endif /* MIKNET_MIKMETA_H_  */
