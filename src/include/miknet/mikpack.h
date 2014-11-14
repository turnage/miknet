#ifndef MIKNET_MIKPACK_H_
#define MIKNET_MIKPACK_H_

#include "miknet/mikmeta.h"

#define MIKFRAG_HEADER_SIZE MIKMETA_SERIALIZED_OCTETS
#define MIKPACK_REAL_FRAG_SIZE 1024
#define MIKPACK_FRAG_SIZE (MIKPACK_REAL_FRAG_SIZE - MIKMETA_SERIALIZED_OCTETS)

typedef struct mikpack_t {
	mikmeta_t metadata;
	uint16_t ref_count;
	uint8_t *data;
} mikpack_t;

/**
 *  Returns an estimate of the memory required for a packet carrying the
 *  inquired amount of octets.
 */
size_t mikpack_mem_est(size_t len);

#endif /* MIKNET_MIKPACK_H_ */
