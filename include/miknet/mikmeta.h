#ifndef MIKNET_MIKMETA_H_
#define MIKNET_MIKMETA_H_

#include <stdint.h>
#include <stddef.h>

#define MIKMETA_SERIALIZED_OCTETS 9

typedef uint8_t mikchan_t;

typedef enum {
	MIK_NONE   = 0,
	MIK_JOIN   = 1,
	MIK_QUIT   = 2,
	MIK_ACKN   = 3,
	MIK_SAFE   = 4,
	MIK_UNSAFE = 5
} miktype_t;

typedef struct mikmeta_t {
	uint16_t id;
	uint16_t part;
	miktype_t type;
	mikchan_t channel;
	uint16_t size;
} mikmeta_t;

/**
 *  Serializes the information in a Miknet meta object to be sent over
 *  the network.
 *
 *  Returns 0 on success.
 */
int mikmeta_serialize(const mikmeta_t *meta, uint8_t *destination);

/**
 *  Deserializes data that represents Miknet meta. The data should be
 *  MIKMETA_SERIALIZED_OCTETS long.
 *
 *  Returns 0 on success.
 */
int mikmeta_deserialize(mikmeta_t *meta, const uint8_t *serialized);

#endif /* MIKNET_MIKMETA_H_  */
