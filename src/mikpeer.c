#include <miknet/miknet.h>

int mikpeer (miknode_t *n)
{
	int sock, i, pos = 0;
	struct sockaddr_storage addr;
	socklen_t addrlen;

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

	return 0;
}

/**
 *  Send data to a peer.
 *
 *  @p: peer to send to
 *  @t: metadata for this packet
 *  @d: data to send
 *  @len: length of the data to send
 *  @m: mode; tcp or udp
 *
 *  @return: 0 on success
 */
int mikpeer_send (mikpeer_t *p, miktype_t t, void *d, size_t len, miknet_t m)
{
	mikcommand_t command = {0};
	command.peer = p->index;
	command.pack = mikpack(t, d, len);
	command.mode = m;

	miklist_add(p->node->commands, &command, sizeof(mikcommand_t));

	return 0;
}

int mikpeer_close (mikpeer_t *p)
{
	close(p->tcp);
	memset(&p->addr, 0, sizeof(struct sockaddr_storage));

	p->state = MIK_DISC;
	p->tcp = 0;
	p->addrlen = 0;
	p->sent = 0;
	p->recvd = 0;

	return 0;
}