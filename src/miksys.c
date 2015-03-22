#include "miknet/miksys.h"

static int mikbind(	const posix_t *pos,
			int sockfd,
			const struct sockaddr *addr,
			socklen_t addrlen)
{
	return bind(sockfd, addr, addrlen);
}

static void mikfreeaddrinfo(const posix_t *pos, struct addrinfo *res)
{
	freeaddrinfo(res);
}

static int mikgetaddrinfo(	const posix_t *pos,
				const char *node,
				const char *service,
				const struct addrinfo *hints,
				struct addrinfo **res)
{
	return getaddrinfo(node, service, hints, res);
}

static ssize_t miksendto(	const posix_t *pos,
				int sockfd,
				const void *buf,
				size_t len,
				int flags,
				const struct sockaddr *dest_addr,
				socklen_t addrlen)
{
	return sendto(sockfd, buf, len, flags, dest_addr, addrlen);
}

static int miksetsockopt(	const posix_t *pos,
				int sockfd,
				int level,
				int optname,
				const void *optval,
				socklen_t optlen)
{
	return setsockopt(sockfd, level, optname, optval, optlen);
}

static int miksocket(const posix_t *pos, int domain, int type, int protocol)
{
	return socket(domain, type, protocol);
}

static ssize_t mikrecvfrom(	const posix_t *pos,
				int sockfd,
				void *buf,
				size_t len,
				int flags,
				struct sockaddr *src_addr,
				socklen_t *addrlen)
{
	return recvfrom(sockfd, buf, len, flags, src_addr, addrlen);
}

posix_t mikposix()
{
	posix_t posix = {	mikbind,
				mikfreeaddrinfo,
				mikgetaddrinfo,
				miksendto,
				miksetsockopt,
				miksocket,
				mikrecvfrom};

	return posix;
}
