#ifndef MIKNET_MIKNODE_H_
#define MIKNET_MIKNODE_H_

#include "miknet/mikaddr.h"
#include "miknet/mikgram.h"
#include "miknet/mikpack.h"
#include "miknet/mikpeer.h"
#include "miknet/miksys.h"

typedef struct miknode_t {
	int sockfd;
	const posix_t *posix;
	mikpeer_t *peers;
	uint8_t max_peers;
	mikgram_t *outgoing;
	mikpack_t *incoming;
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
miknode_t *miknode_create(	const posix_t *posix,
				const mikaddr_t *addr,
				uint16_t port,
				uint8_t max_peers);
miknode_t *miknode(uint16_t port, uint8_t max_peers);

/**
 *  Inserts a foreign node to the local node's contact list. A new one can be
 *  created from a string address and port combination if an exact address is
 *  not known. Returns the index of the new peer.
 */
int miknode_insert_peer(miknode_t *node, const mikaddr_t *addr);
int miknode_new_peer(miknode_t *node, const char *address, uint16_t port);

/**
 *  Sends data to a foreign miknode. Returns 0 on success.
 */
int miknode_send(miknode_t *node, int peer, const void *data, size_t len);

/**
 *  Dequeues pending operations and updates the state of peers. Returns 0 on
 *  success.
 */
int miknode_service(miknode_t *node);

/**
 *  Frees the resources used by a miknode.
 */
void miknode_close(miknode_t *node);

#endif /* MIKNET_MIKNODE_H_ */
