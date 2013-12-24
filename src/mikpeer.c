#include <miknet/miknet.h>

int mikpeer (miknode_t *n)
{
	int sock, i, pos = 0;
	struct sockaddr_storage addr;
	socklen_t addrlen = sizeof(struct sockaddr_storage);

	sock = accept(n->tcp, (struct sockaddr *)&addr, &addrlen);
	if (sock < 0)
		return mik_debug(ERR_SOCKET);

	if (n->peerc >= n->peermax) {
		close(sock);
		return ERR_PEER_MAX;
	}

	n->peerc++;

	for (i = 0; i < n->peermax; ++i) {
		if (n->peers[i].state == MIK_DISC) {
			pos = i;
			break;
		}
	}

	n->peers[pos].node = n;
	n->peers[pos].index = pos;
	n->peers[pos].state = MIK_CONN;
	n->peers[pos].tcp = sock;
	n->peers[pos].addr = addr;
	n->peers[pos].addrlen = addrlen;
	n->peers[pos].sent = 0;
	n->peers[pos].recvd = 0;
	n->fds[2 + pos].fd = sock;
	n->fds[2 + pos].events = POLLIN;

	return 0;
}

/**
 *  Connect to an address.
 *
 *  @n: node
 *  @a: address to connect to
 *  @p: port to connect on
 *
 *  @return: index of the new peer
 */
int mikpeer_connect(miknode_t *n, const char *a, uint16_t p)
{
	if (n->peerc >= n->peermax)
		return ERR_WOULD_FAULT;

	int err, sock, yes = 1;
	int pos = 0, j;
	struct addrinfo hint = {0}, *li, *i;
	char portstr[MIK_PORT_MAX] = {0};
	sprintf(portstr, "%u", p);

	if (n->ip == MIK_IPV4)
		hint.ai_family = AF_INET;
	else if (n->ip == MIK_IPV6)
		hint.ai_family = AF_INET6;

	hint.ai_socktype = SOCK_STREAM;

	if (!a)
		hint.ai_flags = AI_PASSIVE;

	sock = socket(hint.ai_family, SOCK_STREAM, 0);
	setsockopt(sock, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));

	err = getaddrinfo(a, portstr, &hint, &li);
	if (err < 0)
		return mik_debug(ERR_ADDRESS);

	for (i = li; i; i = i->ai_next) {
		err = connect(sock, i->ai_addr, i->ai_addrlen);
		if (!err)
			break;
	}

	freeaddrinfo(li);

	if (err)
		return mik_debug(ERR_CONNECT);

	for (j = 0; j < n->peermax; ++j) {
		if (n->peers[j].state == MIK_DISC) {
			pos = j;
			break;
		}
	}

	n->peers[pos].node = n;
	n->peers[pos].index = pos;
	n->peers[pos].state = MIK_CONN;
	n->peers[pos].tcp = sock;
	n->peers[pos].addrlen = i->ai_addrlen;
	n->peers[pos].sent = 0;
	n->peers[pos].recvd = 0;
	n->fds[1 + pos].fd = sock;
	n->fds[1 + pos].events = POLLIN;

	memcpy(&n->peers[pos].addr, i->ai_addr, i->ai_addrlen);

	return pos;
}

/**
 *  Send data to a peer.
 *
 *  @p: peer to send to
 *  @t: metadata for this packet
 *  @d: data to send
 *  @len: length of the data to send
 *
 *  @return: 0 on success
 */
int mikpeer_send (mikpeer_t *p, miktype_t t, void *d, size_t len)
{
	mikevent_t command = {0};
	command.peer = p->index;
	command.pack = mikpack(t, d, len);

	miklist_add(p->node->commands, &command, sizeof(mikevent_t));

	return 0;
}

int mikpeer_close (mikpeer_t *p)
{
	p->node->fds[1 + p->index].fd = -1;

	close(p->tcp);
	memset(&p->addr, 0, sizeof(struct sockaddr_storage));

	p->state = MIK_DISC;
	p->tcp = 0;
	p->addrlen = 0;
	p->sent = 0;
	p->recvd = 0;

	return 0;
}
