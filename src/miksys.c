#include "miknet/miksys.h"

static syswrapper_t system = {	freeaddrinfo,
				getaddrinfo,
				setsockopt,
				socket};

void miksys_remap(syswrapper_t wrapper) { system = wrapper; }

void mikfreeaddrinfo(struct addrinfo *res) { system.freeaddrinfo(res); }

int mikgetaddrinfo(	const char *node,
			const char *service,
			const struct addrinfo *hints,
			struct addrinfo **res)
{
	return system.getaddrinfo(node, service, hints, res);
}

int miksetsockopt(	int sockfd,
			int level,
			int optname,
			const void *optval,
			socklen_t optlen)
{
	return system.setsockopt(sockfd, level, optname, optval, optlen);
}

int miksocket(int domain, int type, int protocol)
{
	return system.socket(domain, type, protocol);
}
