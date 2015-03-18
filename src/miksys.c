#include "miknet/miksys.h"

static void mikbind(	posix_t *pos,
			int sockfd,
			const struct sockaddr *addr,
			socklen_t addrlen)
{
	return bind(sockfd, addr, addrlen);
}

static void mikfreeaddrinfo(posix_t *pos, struct addrinfo *res)
{
	freeaddrinfo(res);
}

static int mikgetaddrinfo(	posix_t *pos,
				const char *node,
				const char *service,
				const struct addrinfo *hints,
				struct addrinfo **res)
{
	return getaddrinfo(node, service, hints, res);
}

static int miksend(	posix_t *pos,
			int sockfd,
			const void *buf,
			size_t len,
			int flags)
{
	return send(sockfd, buf, len, flags);
}

static int miksetsockopt(	posix_t *pos,
				int sockfd,
				int level,
				int optname,
				const void *optval,
				socklen_t optlen)
{
	return setsockopt(sockfd, level, optname, optval, optlen);
}

static int miksocket(posix_t *pos, int domain, int type, int protocol)
{
	return socket(domain, type, protocol);
}

static ssize_t mikrecv(	posix_t *pos,
			int sockfd,
			void *buf,
			size_t len,
			int flags)
{
	return recv(sockfd, buf, len, flags);
}

posix_t mikposix()
{
	posix_t posix = {	mikbind,
				mikfreeaddrinfo,
				mikgetaddrinfo,
				miksend,
				miksetsockopt,
				miksocket,
				mikrecv};

	return posix;
}
