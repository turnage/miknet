#include <miknet/miknet.h>

static int mik_sock (int *t, struct addrinfo *h)
{
	int err = 0;
	int yes = 1;

	*t = socket(h->ai_family, SOCK_STREAM, 0);
	if (*t < 0)
		return mik_debug(ERR_SOCKET);

	err = setsockopt(*t, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	if (err < 0)
		return mik_debug(ERR_SOCK_OPT);

	return 0;
}

static int mik_testbind (int s, struct addrinfo *h, const char *p)
{
	int err = 0;
	int bound = 0;
	struct addrinfo *li = NULL;
	struct addrinfo *i = NULL;
	struct addrinfo c = *h;

	err = getaddrinfo(NULL, p, &c, &li);
	
	if (err < 0)
		return mik_debug(ERR_ADDRESS);
	
	for (i = li; i; i = i->ai_next) {
		err = bind(s, i->ai_addr, i->ai_addrlen);
		if (!err) {
			bound = 1;
			break;
		}
	}

	freeaddrinfo(li);

	return bound;
}

/**
 *  Provided with a detaild request, bind two sockets to the same port number
 *  (on different protocols).
 *
 *  @t: SOCK_STREAM socket
 *  @h: copy of address request
 *  @p: port or 0 for auto-assign
 *
 *  @return: the port bound to
 */
static int mik_bind (int *t, struct addrinfo h, uint16_t p)
{
	char portstr[MIK_PORT_MAX] = {0};

	sprintf(portstr, "%u", p);
	mik_sock(t, &h);

	return mik_testbind(*t, &h, portstr);
}

/**
 *  Create a miknode on the network level. It does not need to be ready for use,
 *  only ready for configuration.
 *
 *  @n: the node
 *  @ip: IP type, 4 or 6
 *  @port: requested port or 0 for autoassign
 *
 *  @return: 0 on success
 */
int miknode (miknode_t *n, mikip_t ip, uint16_t port)
{
	if (!n)
		return ERR_MISSING_PTR;

	struct addrinfo hint = {0};

	n->ip = ip;

	if (n->ip == MIK_IPV4)
		hint.ai_family = AF_INET;
	else if (n->ip == MIK_IPV6)
		hint.ai_family = AF_INET6;

	hint.ai_flags = AI_PASSIVE;
	hint.ai_socktype = SOCK_STREAM;

	mik_bind(&n->tcp, hint, port);

	return 0;
}

/**
 *  Prepare a miknode for use.
 *
 *  @peers: maximum amount of peers
 *  @up: up bandwidth limit (bytes/sec)
 *  @down: down bandwidth limit (bytes/sec)
 *
 *  @return: 0 on success
 */
int miknode_config (miknode_t *n, uint16_t peers, uint32_t up, uint32_t down)
{
	if (!n)
		return ERR_MISSING_PTR;

	n->peermax = peers;
	n->peerc = 0;
	n->upcap = up;
	n->downcap = down;

	n->peers = calloc(n->peermax, sizeof(mikpeer_t));
	n->fds = calloc(n->peermax + 1, sizeof(mikpeer_t));
	if (!n->peers || !n->fds)
		return mik_debug(ERR_MEMORY);

	n->fds[0].fd = n->tcp;
	n->fds[0].events = POLLIN;
	n->packs = NULL;
	n->commands = NULL;

	listen(n->tcp, n->peermax);

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
int miknode_connect(miknode_t *n, const char *a, uint16_t p)
{
	if (n->peerc >= n->peermax)
		return ERR_WOULD_FAULT;

	int err = 0;
	int sock = 0;
	int yes = 1;
	int pos = -1;
	int j = 0;
	struct addrinfo hint = {0};
	struct addrinfo *li = NULL;
	struct addrinfo *i = NULL;
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

	if (pos >= 0) {
		n->peers[pos].node = n;
		n->peers[pos].index = pos;
		n->peers[pos].state = MIK_CONN;
		n->peers[pos].tcp = sock;
		n->peers[pos].sent = 0;
		n->peers[pos].recvd = 0;
		n->fds[1 + pos].fd = sock;
		n->fds[1 + pos].events = POLLIN;
	}

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
int miknode_send (mikpeer_t *p, ref *d, size_t len, uint32_t channel)
{
	if (len > MIK_PACK_MAX)
		return ERR_WOULD_FAULT;

	miklist_t command = {0};
	miklist_t *cmds = p->node->commands;
	command.pack = mikpack(MIK_DATA, d, len, channel);
	command.pack.peer = p->index;

	p->node->commands = miklist_add(cmds, &command);

	return 0;
}

/**
 *  Receive data from a peer.
 *
 *  @p: peer to receive from
 *
 *  @return: 0 on success
 */
static int miknode_recv (mikpeer_t *p)
{
	mikpack_t pack = {0};
	miklist_t *e = p->node->packs;
	int size = recv(p->tcp, &pack, sizeof(mikpack_t), MSG_PEEK);

	if (size < 0)
		mik_debug(ERR_SOCKET);

	if (!size) {
		/* peer disconnected */
		recv(p->tcp, NULL, 0, 0);
		miklist_t event = {0};
		event.pack.peer = p->index;
		event.pack.type = MIK_QUIT;
		p->node->packs = miklist_add(e, &event);
		mikpeer_close(p);
	} else {
		if (pack.len > MIK_PACK_MAX)
			return ERR_WOULD_FAULT;

		recv(p->tcp, &pack, sizeof(mikpack_t), 0);
		char *buffer = calloc(1, pack.len);
		recv(p->tcp, buffer, pack.len, 0);

		miklist_t event= {0};
		event.pack = pack;
		event.pack.peer = p->index;
		event.pack.data = (void *)buffer;

		p->node->packs = miklist_add(e, &event);
		p->recvd += sizeof(mikpack_t) + pack.len;
	}

	return 0;
}

/**
 *  Service the node. Execute commands in the queue and add incoming events.
 *
 *  @n: the node
 *  @t: time, in milliseconds
 *
 *  @return: the number of events to be handled
 */
int miknode_poll (miknode_t *n, int t)
{
	if (!n)
		return ERR_MISSING_PTR;

	int i = 0;
	int events = 0;
	int err = poll(n->fds, 1 + n->peermax, t);

	/* Connection on master TCP socket. */
	if (n->fds[0].revents & POLLIN) {
		err = mikpeer(n);
		if (err < 0)
			mik_debug(err);
	}

	for (i = 0; i < n->peermax; ++i) {
		if (n->fds[1 + i].revents & POLLIN) {
			miknode_recv(&n->peers[i]);
			n->fds[1 + i].revents = 0;
			events++;
		}
	}

	while (n->commands) {
		int sock = n->peers[n->commands->pack.peer].tcp;
		void *data = (void *)n->commands->pack.data;
		int length = sizeof(mikpack_t) + n->commands->pack.len;
		char buffer[length];

		memset(buffer, 0, length);
		memcpy(buffer, &n->commands->pack, sizeof(mikpack_t));
		memcpy(buffer + sizeof(mikpack_t), data, n->commands->pack.len);

		int sent = send(sock, buffer, length, 0);
		n->peers[n->commands->pack.peer].sent += sent;

		n->commands = miklist_next(n->commands);
	}

	return events;
}

/**
 *  Free all the resources used by a miknode.
 *
 *  @n: the miknode
 */
void miknode_close (miknode_t *n)
{
	miklist_close(n->commands);
	miklist_close(n->packs);

	int i;
	for (i = 0; i < n->peermax; ++i)
		if (n->peers[i].state == MIK_CONN)
			mikpeer_close(&n->peers[i]);

	free(n->fds);
	free(n->peers);

	close(n->tcp);
}
