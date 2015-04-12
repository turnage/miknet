#ifndef MIKNET_MIKPACK_H_
#define MIKNET_MIKPACK_H_

#include "miknet/mikaddr.h"
#include "miknet/mikgram.h"

typedef struct mikmsg_t {
	void *data;
	size_t len;
	uint8_t peer;
	mikaddr_t addr;
	struct mikmsg_t *next;
} mikmsg_t;

/**
 *  Creates a mikmsg from a mikgram(s). mikmsgs must be destroyed with
 *  mikmsg_close. The mikgram is still owned by caller.
 */
mikmsg_t *mikmsg(const mikgram_t *gram, const mikaddr_t *addr);

/**
 *  Frees the resources used by a mikmsg.
 */
void mikmsg_close(mikmsg_t *msg);

#endif /* MIKNET_MIKPACK_H_ */
