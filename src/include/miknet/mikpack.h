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
 *  Returns an estimate of the memory required for a packet carrying the
 *  inquired amount of octets.
 */
size_t mikpack_mem_est(size_t len);

/**
 *  Creates a mikpack from the provided data. The provided destination pointer
 *  **must** point to memory that mikpack can write to. The size of that memory
 *  must be enough to hold the packet. Find out how much is needed by calling
 *  mikpack_mem_est() on the size of the data to be sent.
 */
int mikpack(	mikpack_t *pack,
		mikflag_t flags,
		const uint8_t *src,
		size_t len,
		uint8_t *dest);

/**
 *  Returns a pointer to the beginning of a specific fragment in the mikpack.
 */
int mikpack_frag(const mikpack_t *pack, uint16_t fragment, mikmeta_t *metadata);

/**
 *  Returns a pointer to the beginning of a specific fragment's data in the
 *  mikpack.
 */
uint8_t *mikpack_frag_data(const mikpack_t *pack, uint16_t fragment);

#endif /* MIKNET_MIKPACK_H_ */
