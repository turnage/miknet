#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>

#include "miknet/miknode.h"

#include "miknet/mikdef.h"
#include "miknet/miktime.h"

/**
 *  Dequeues a single miknet datagram waiting to be received.
 */
static int miknode_dequeue_incoming(miknode_t *node)
{
	mikaddr_t addr;
	mikgram_t *gram;
	mikmsg_t *nav;
	mikmsg_t **next;
	ssize_t poll_err;
	ssize_t check_err;

	poll_err = mikstation_poll(node->sockfd, node->posix);
	if (poll_err == MIKERR_NO_MSG)
		return MIK_SUCCESS;
	else if (poll_err < 0)
		return MIKERR_NET_FAIL;
	else if (poll_err == 0 || poll_err > MIKNET_GRAM_MAX_SIZE) {
		mikstation_discard(node->sockfd, node->posix);
		return MIKERR_NONCONFORM;
	}

	if (mikstation_receive(	node->sockfd,
				node->posix,
				&gram,
				&addr) != MIK_SUCCESS)
		return MIKERR_NET_FAIL;

	if (node->incoming == NULL)
		next = &node->incoming;
	else {
		for (nav = node->incoming; nav->next != NULL; nav = nav->next);
		next = &nav->next;
	}

	*next = mikmsg(gram, &addr);
	if (*next == NULL)
		return MIKERR_NONCONFORM;

	return MIK_SUCCESS;
}

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
	err = mikstation_send(	node->sockfd,
				node->posix,
				node->outgoing,
				&node->peers[gram->peer].address);

	node->outgoing = node->outgoing->next;
	free(gram);

	return err;
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

	if (peer >= node->max_peers || peer < 0)
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
	uint64_t end;
	int err;

	if (node == NULL)
		return MIKERR_BAD_PTR;

	end = miktime() + nanoseconds;
	err = MIK_SUCCESS;

	while (miktime() < end) {
		err = miknode_dequeue_outgoing(node);
		if (err != MIK_SUCCESS)
			return err;

		err = miknode_dequeue_incoming(node);
		if (err == MIKERR_NET_FAIL)
			return err;

		miktime_sleep(nanoseconds / 10);
	}

	return err;
}

void miknode_close(miknode_t *node)
{
	mikgram_close(node->outgoing);
	mikmsg_close(node->incoming);
	free(node);
}
