#include <miknet/miknet.h>

int mikpeer (miknode_t *n)
{
	int sock = 0;
	int i = 0;
	int pos = 0;

	sock = accept(n->tcp, NULL, NULL);
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
	n->peers[pos].sent = 0;
	n->peers[pos].recvd = 0;
	n->fds[1 + pos].fd = sock;
	n->fds[1 + pos].events = POLLIN;

	mikpack_t join = mikpack(MIK_JOIN, NULL, 0, 0);
	join.peer = pos;
	n->packs = mikvec_add(n->packs, join);

	return 0;
}

int mikpeer_close (mikpeer_t *p)
{
	p->node->fds[1 + p->index].fd = 0;

	close(p->tcp);

	p->state = MIK_DISC;
	p->tcp = 0;
	p->sent = 0;
	p->recvd = 0;

	return 0;
}
