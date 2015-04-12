#ifndef MIKNET_MIKADDR_H_
#define MIKNET_MIKADDR_H_

#include <stdint.h>
#include <sys/types.h>

#include "miknet/miksys.h"

/**
 *  mikaddr is a passive structure containing the information necessary to
 *  communicate with a peer.
 */
typedef struct mikaddr_t {
	struct sockaddr addr;
	socklen_t addrlen;
} mikaddr_t;

/**
*  Creates a mikaddr instance from an address, port, and ip address type.
*
*  If the mikaddr's connectable flag is not true, creating the address failed.
*/
int mikaddr(	mikaddr_t *mikaddr,
		const mikposix_t *pos,
		const char *addr,
		uint16_t port);

#endif /* MIKNET_MIKADDR_H_ */
