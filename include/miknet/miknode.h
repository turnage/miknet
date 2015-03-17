#ifndef MIKNET_MIKNODE_H_
#define MIKNET_MIKNODE_H_

#include "miknet/mikaddr.h"
#include "miknet/mikpeer.h"
#include "miknet/miksys.h"

typedef struct miknode_t {
	int sockfd;
	posix_t *posix;
	mikpeer_t *peers;
	uint8_t max_peers;
} miknode_t;

/**
 *  Creates and returns a pointer to a miknode, bound to the port requested, or
 *  NULL on failure. The miknode must be closed with miknode_close.
 *
 *  A shortcut is provided for users which takes care of the posix function
 *  wrapper and address generation.
 *
 *  Request a port of 0 for auto-assign.
 */
miknode_t *miknode_create(	posix_t *posix,
				const mikaddr_t *addr,
				uint16_t port,
				uint8_t max_peers);
miknode_t *miknode(uint16_t port, uint8_t max_peers);

/**
 *  Frees the resources used by a miknode.
 */
void miknode_close(miknode_t *node);

#endif /* MIKNET_MIKNODE_H_ */
