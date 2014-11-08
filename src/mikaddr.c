#include <stdio.h>

#include "miknet/mikdef.h"
#include "miknet/mikaddr.h"

int mikaddr(mikaddr_t *mikaddr, posix_t *pos, const char *addr, uint16_t port)
{
	struct addrinfo *candidates = NULL;
	int error = 0;
	struct addrinfo hint = {0};
	char port_string[6] = {0};

	if (!mikaddr || !addr || !pos)
		return MIKERR_BAD_PTR;

	sprintf(port_string, "%u", port);
	hint.ai_family = AF_INET;
	hint.ai_socktype = SOCK_STREAM;

	error = pos->getaddrinfo(pos, addr, port_string, &hint, &candidates);
	if (error)
		return MIKERR_LOOKUP;

	mikaddr->hint = hint;
	mikaddr->candidates = candidates;

	return MIKERR_NONE;
}

void mikaddr_close(mikaddr_t *mikaddr, posix_t *pos)
{
	if (!mikaddr || !pos)
		return;

	pos->freeaddrinfo(pos, mikaddr->candidates);
}
