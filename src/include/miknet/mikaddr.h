#ifndef MIKNET_MIKADDR_H_
#define MIKNET_MIKADDR_H_

#include <stdint.h>
#include <sys/types.h>

#include "miknet/miksys.h"

typedef struct mikaddr_t {
	struct addrinfo hint;
	struct addrinfo *candidates;
} mikaddr_t;

/**
*  Creates a mikaddr instance from an address, port, and ip address type.
*
*  If the mikaddr's connectable flag is not true, creating the address failed.
*/
int mikaddr(mikaddr_t *mikaddr, posix_t *pos, const char *addr, uint16_t port);

/**
*  Attempts to connect to the mikaddr. If it succeeds, the socket fd will be
*  returned.
*
*  A value less than 0 indicates an error.
*/
int mikaddr_connect(const mikaddr_t *mikaddr, posix_t *pos);

/**
* Cleans up resources used by a mikaddr instance.
*/
void mikaddr_close(mikaddr_t *mikaddr, posix_t *pos);

#endif /* MIKNET_MIKADDR_H_ */
