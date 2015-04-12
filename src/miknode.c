#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>

#include "miknet/miknode.h"

#include "miknet/mikdef.h"
#include "miknet/miktime.h"

/**
 *  Dequeues a single outgoing command in the miknode's queue.
 */
static int miknode_dequeue_outgoing(miknode_t *node)
{
	mikgram_t *gram;
	int err;

	if (node == NULL)
		return MIKERR_BAD_PTR;

	if (node->outgoing == NULL)
		return MIK_SUCCESS;

	gram = node->outgoing;
	node->outgoing = node->outgoing->next;

	err = mikstation_send(	node->sockfd,
				node->posix,
				gram,
				&node->peers[gram->peer].address);

	mikgram_close(gram);

	return err;
}

/**
 *  Free all mikpacks in queue.
 */
static void miknode_free_grams(mikgram_t *gram)
{
	if (gram == NULL)
		return;

	miknode_free_grams(gram->next);
	mikgram_close(gram);
}

miknode_t *miknode_create(	const mikposix_t *posix,
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
	node->outgoing = NULL;
	node->incoming = NULL;

	return node;
}

miknode_t *miknode(uint16_t port, uint8_t max_peers)
{
	mikposix_t *posix = mikposix();
	mikaddr_t addr;
	miknode_t *node;

	if (mikaddr(&addr, posix, NULL, port) != MIK_SUCCESS)
		return NULL;

	node = miknode_create(posix, &addr, port, max_peers);
	fcntl(node->sockfd, F_SETFL, fcntl(node->sockfd, F_GETFL) |O_NONBLOCK);

	return node;
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

	if (mikaddr(&addr, node->posix, address, port) != MIK_SUCCESS)
		return MIKERR_NET_FAIL;

	return miknode_insert_peer(node, &addr);
}

int miknode_send(miknode_t *node, int peer, const void *data, size_t len)
{
	mikgram_t *gram;

	if (node == NULL || data == NULL)
		return MIKERR_BAD_PTR;

	if (peer >= node->max_peers || len > MIKNET_MAX_PAYLOAD_SIZE)
		return MIKERR_BAD_VALUE;

	gram = mikgram(data, len);
	if (gram == NULL)
		return MIKERR_BAD_VALUE;
	gram->peer = peer;

	if (node->outgoing == NULL)
		node->outgoing = gram;
	else {
		mikgram_t *nav;
		for (nav = node->outgoing; nav->next != NULL; nav = nav->next);
		nav->next = gram;
	}

	return MIK_SUCCESS;
}

int miknode_service(miknode_t *node, uint64_t nanoseconds)
{
	mikgram_t *gram;
	uint64_t start;
	int err;

	if (node == NULL)
		return MIKERR_BAD_PTR;

	start = miktime();

	while (node->outgoing != NULL && miktime() - start < nanoseconds) {
		err = miknode_dequeue_outgoing(node);
		if (err != MIK_SUCCESS)
			return err;
	}

	return err;
}

void miknode_close(miknode_t *node)
{
	if (node->outgoing != NULL)
		miknode_free_grams(node->outgoing);

	free(node);
}
