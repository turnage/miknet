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

int miknode_insert_peer(miknode_t *node, const mikaddr_t *addr)
{
	if (node == NULL || addr == NULL)
		return MIKERR_BAD_PTR;

	int i = 0;

	while (i < node->max_peers && node->peers[i].exists == MIK_TRUE)
		++i;

	node->peers[i].address = *addr;
	node->peers[i].exists = MIK_TRUE;

	return i;
}

int miknode_new_peer(miknode_t *node, const char *address, uint16_t port)
{
	mikaddr_t addr;

	if (node == NULL || address == NULL)
		return MIKERR_BAD_PTR;

	if (mikaddr(&addr, node->posix, address, port) != MIKERR_NONE)
		return MIKERR_LOOKUP;

	return miknode_insert_peer(node, &addr);
}

int miknode_send(miknode_t *node, int peer, const mikgram_t *gram)
{
	ssize_t sent;

	if (node == NULL || gram == NULL)
		return MIKERR_BAD_PTR;

	if (gram->data == NULL)
		return MIKERR_BAD_PTR;

	if (gram->len == 0)
		return MIKERR_BAD_LENGTH;

	if (node->peers[peer].exists == MIK_FALSE)
		return MIKERR_BAD_PEER;

	sent = node->posix->sendto(	node->posix,
					node->sockfd,
					gram->data,
					gram->len,
					0,
					&node->peers[peer].address.addr,
					node->peers[peer].address.addrlen);

	if (sent != gram->len)
		return MIKERR_BAD_SEND;

	return 0;
}

void miknode_close(miknode_t *node)
{
	free(node);
}
