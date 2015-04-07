#ifndef MIKNET_MIKSTATION_H_
#define MIKNET_MIKSTATION_H_

#include "miknet/mikaddr.h"
#include "miknet/mikgram.h"
#include "miknet/miksys.h"

/**
 *  The mikstation module handles any network inferfaces which lie beneath the
 *  miknet protocol.
 */

/**
 *  Clears one datagram waiting to be received. Returns the number of octets
 *  discarded.
 */
ssize_t mikstation_discard(const int sockfd, const posix_t *posix);

/**
 *  Returns the size of the next datagram waiting to be received. Nonnegative
 *  values indicate success.
 */
ssize_t mikstation_poll(const int sockfd, const posix_t *posix);

/**
 *  Receives a single mikgram from the network and writes both the gram and
 *  sender address to the provided arguments. Returns 0 on success.
 */
int mikstation_receive(	const int sockfd,
			const posix_t *posix,
			mikgram_t *gram,
			mikaddr_t *addr);

/**
 *  Sends a single mikgram over the network to the given address. Returns 0 on
 *  success.
 */
int mikstation_send(	const int sockfd,
			const posix_t *posix,
			const mikgram_t *gram,
			const mikaddr_t *addr);

#endif /* MIKNET_MIKSTATION_H_ */
