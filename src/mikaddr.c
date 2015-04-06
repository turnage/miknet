#include <stdio.h>
#include <netdb.h>

#include "miknet/mikdef.h"
#include "miknet/mikaddr.h"

static struct addrinfo *mikaddr_get_candidate(	const posix_t *pos,
						const char *addr,
						uint16_t port)
{
	struct addrinfo *candidate;
	struct protoent *udp;
	int error;
	struct addrinfo hint = {0};
	char port_string[6] = {0};

	udp = getprotobyname("udp");
	hint.ai_protocol = udp->p_proto;
	endprotoent();

	/* Request INADDR_ANY if no address specified. */
	if (addr == NULL)
		hint.ai_flags = AI_PASSIVE;

	hint.ai_family = AF_INET;
	hint.ai_socktype = SOCK_DGRAM;

	sprintf(port_string, "%u", port);
	error = pos->getaddrinfo(pos, addr, port_string, &hint, &candidate);
	if (error)
		return NULL;

	return candidate;
}

int mikaddr(	mikaddr_t *mikaddr,
		const posix_t *pos,
		const char *addr,
		uint16_t port)
{
	struct addrinfo *candidate;

	if (mikaddr == NULL || pos == NULL)
		return MIKERR_BAD_PTR;

	candidate = mikaddr_get_candidate(pos, addr, port);

	if (candidate == NULL)
		return MIKERR_NET_FAIL;

	mikaddr->addr = *candidate->ai_addr;
	mikaddr->addrlen = candidate->ai_addrlen;

	pos->freeaddrinfo(pos, candidate);

	return MIKERR_NONE;
}
