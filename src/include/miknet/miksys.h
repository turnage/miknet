#ifndef MIKNET_MIKSYS_H_
#define MIKNET_MIKSYS_H_

#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>

/**
 *  miksys provides call guards for the posix calls the miknet library makes. By
 *  default, they simply forward to the posix function. Using miksys_remap, they
 *  they can be set to call other functions for testing or extra safety.
 */

typedef struct syswrapper_t {
	void (*freeaddrinfo)(struct addrinfo *);
	int (*getaddrinfo)(	const char *,
				const char *,
				const struct addrinfo *,
				struct addrinfo **);
	int (*setsockopt)(int, int, int, const void *, socklen_t);
	int (*socket)(int, int, int);
} syswrapper_t;

/**
 *  Remaps miksys with the function definitions it needs. If this is not
 *  called, the defaults are used.
 */
void miksys_remap(syswrapper_t wrapper);

void mikfreeaddrinfo(struct addrinfo *res);

int mikgetaddrinfo(	const char *node,
			const char *service,
			const struct addrinfo *hints,
			struct addrinfo **res);

int miksetsockopt(	int sockfd,
			int level,
			int optname,
			const void *optval,
			socklen_t optlen);

int miksocket(int domain, int type, int protocol);

#endif /* MIKNET_MIKSYS_H_ */
