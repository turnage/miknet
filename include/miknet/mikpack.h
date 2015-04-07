#ifndef MIKNET_MIKPACK_H_
#define MIKNET_MIKPACK_H_

#include "miknet/mikaddr.h"

typedef struct mikpack_t {
	void *data;
	size_t len;
	uint8_t peer;
	mikaddr_t addr;
	struct mikpack_t *next;
} mikpack_t;

#endif /* MIKNET_MIKPACK_H_ */
