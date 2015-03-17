#ifndef MIKNET_MIKSYS_H_
#define MIKNET_MIKSYS_H_

#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>

/**
 *  miksys provides wrappers for functions provided by the kernel or C language
 *  that parts of miknet might otherwise call directly.
 *
 *  This layer of misdirection allows the functions to be easily switched out
 *  with other dependencies at runtime, for testing.
 */

typedef struct posix_t {
	int (*bind)(	struct posix_t *posix,
			int,
			const struct sockaddr *,
			socklen_t);
	void (*freeaddrinfo)(struct posix_t *, struct addrinfo *);
	int (*getaddrinfo)(	struct posix_t *,
				const char *,
				const char *,
				const struct addrinfo *,
				struct addrinfo **);
	int (*setsockopt)(	struct posix_t *,
				int,
				int,
				int,
				const void *,
				socklen_t);
	int (*socket)(struct posix_t *, int, int, int);
} posix_t;

/**
 *  Returns the default posix wrapper.
 */
posix_t mikposix();

#endif /* MIKNET_MIKSYS_H_ */
