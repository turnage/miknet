#ifndef MIKNET_MIKPACK_H_
#define MIKNET_MIKPACK_H_

#include "miknet/mikmeta.h"

#define MIKFRAG_HEADER_SIZE MIKMETA_SERIALIZED_OCTETS
#define MIKPACK_REAL_FRAG_SIZE 512
#define MIKPACK_FRAG_SIZE (MIKPACK_REAL_FRAG_SIZE - MIKMETA_SERIALIZED_OCTETS)

typedef struct mikpack_t {
	uint8_t *data;
	uint16_t frags;
	uint16_t ref_count;
} mikpack_t;

/**
 *  Creates a mikpack from the provided data.
 */
int mikpack(	mikpack_t *pack,
		miktype_t type,
		const uint8_t *src,
		size_t len);

/**
 *  Deserializes the data in the requested fragment into the passed mikmeta_t.
 */
int mikpack_frag(const mikpack_t *pack, uint16_t fragment, mikmeta_t *metadata);

/**
 *  Returns a pointer to the beginning of a specific fragment's data in the
 *  mikpack.
 */
uint8_t *mikpack_frag_data(const mikpack_t *pack, uint16_t fragment);

/**
 *  Frees the resources used by a mikpack_t.
 */
void mikpack_close(mikpack_t *pack);

#endif /* MIKNET_MIKPACK_H_ */
