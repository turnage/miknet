#include <stdlib.h>
#include <sys/socket.h>

#include "miknet/miknode.h"
#include "miknet/mikdef.h"

miknode_t *miknode_create(	const posix_t *posix,
				const mikaddr_t *addr,
				uint16_t port,
				uint8_t max_peers)
{
	miknode_t *node;
	int sockfd;
	int optval = 1;

	if (posix == NULL)
		return NULL;

	sockfd = posix->socket(posix, AF_INET, SOCK_DGRAM, 0);
	if (sockfd == -1)
		return NULL;

	posix->setsockopt(	posix,
				sockfd,
				SOL_SOCKET,
				SO_REUSEADDR,
				&optval,
				sizeof(int));

	if (posix->bind(posix, sockfd, &addr->addr, addr->addrlen) != 0)
		return NULL;


	node = malloc(sizeof(miknode_t) + sizeof(mikpeer_t) * max_peers);
	if (node == NULL)
		return NULL;

	node->peers = (void *)node + sizeof(miknode_t);
	node->max_peers = max_peers;
	node->posix = posix;
	node->sockfd = sockfd;

	return node;
}

miknode_t *miknode(uint16_t port, uint8_t max_peers)
{
	posix_t *posix = mikposix();
	mikaddr_t addr;

	if (mikaddr(&addr, posix, NULL, port) != MIKERR_NONE)
		return NULL;

	return miknode_create(posix, &addr, port, max_peers);
}

void miknode_close(miknode_t *node)
{
	free(node);
}
