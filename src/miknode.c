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

static int miknode_config (miknode_t *n, uint16_t peers)
{
        if (!n)
                return ERR_MISSING_PTR;

        n->peermax = peers;
        n->peerc = 0;

        n->peers = calloc(n->peermax, sizeof(mikpeer_t));
        n->fds = calloc(n->peermax + 1, sizeof(mikpeer_t));
        if (!n->peers || !n->fds)
                return mik_debug(ERR_MEMORY);

        n->fds[0].fd = n->tcp;
        n->fds[0].events = POLLIN;
        memset(&n->packs, 0, sizeof(mikvec_t));
        memset(&n->commands, 0, sizeof(mikvec_t));

        listen(n->tcp, n->peermax);

        return 0;
}

/**
 *  Create a miknode on the network level. It needs to be ready to use.
 *
 *  @n: the node
 *  @ip: IP type, 4 or 6
 *  @port: requested port or 0 for autoassign
 *
 *  @return: the port the node listens on, or an error less than 0
 */
int miknode (miknode_t *n, mikip_t ip, uint16_t port, uint16_t peers)
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

	int ret = mik_bind(&n->tcp, hint, port);

	miknode_config(n, peers);

	return ret;
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
	if (!n)
		return ERR_MISSING_PTR;

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

	n->peerc++;

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
	if (!p || !d)
		return ERR_MISSING_PTR;

	if (len > MIK_PACK_MAX)
		return ERR_WOULD_FAULT;

	if (p->state == MIK_DISC)
		return ERR_CONNECT;

	mikpack_t command = mikpack(MIK_DATA, d, len, channel);
	command.peer = p->index;

	p->node->commands = mikvec_add(p->node->commands, command);

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
	if (!p)
		return ERR_MISSING_PTR;

	char buffer[MIK_META_SZ] = {0};
	int size = recv(p->tcp, buffer, MIK_META_SZ, MSG_PEEK);
	mikmeta_t data = mik_read_meta(buffer);

	if (size < 0) {
		mik_debug(ERR_SOCKET);
	} else if (!size) {
		/* peer disconnected */
		recv(p->tcp, NULL, 0, 0);
		mikpack_t event = {0};
		event.peer = p->index;
		event.type = MIK_QUIT;
		p->node->packs = mikvec_add(p->node->packs, event);
		p->node->peerc--;
		mikpeer_close(p);
	} else {
		if (data.len > MIK_PACK_MAX)
			return ERR_WOULD_FAULT;

		recv(p->tcp, buffer, MIK_META_SZ, 0);

		void *tmp = try_alloc(NULL, data.len);
		recv(p->tcp, tmp, data.len, 0);

		mikpack_t event = {0};
		event.type = data.type;
		event.channel = data.channel;
		event.len = data.len;
		event.peer = p->index;
		event.data = tmp;

		p->node->packs = mikvec_add(p->node->packs, event);
		p->recvd += MIK_META_SZ + event.len;
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

	i = 0;
	while (i < n->commands.size) {
		int sock = n->peers[n->commands.data[i].peer].tcp;
		void *data = (void *)n->commands.data[i].data;
		int len = n->commands.data[i].len;
		char buffer[MIK_META_SZ + len];

		mik_write_meta(n->commands.data[i], buffer);
		memcpy(buffer + MIK_META_SZ, n->commands.data[i].data, len);

		int sent = send(sock, buffer, MIK_META_SZ + len, 0);
		if (sent < 0)
			mik_debug(ERR_SOCKET);

		n->peers[n->commands.data[i].peer].sent += sent;

		i++;
	}

	n->commands = mikvec_clear(n->commands);

	n->commands.rs_mall++;
	n->packs.rs_mall++;

	return events;
}

/**
 *  Free all the resources used by a miknode.
 *
 *  @n: the miknode
 */
void miknode_close (miknode_t *n)
{
	if (!n)
		return;

	n->commands = mikvec_close(n->commands);
	n->packs = mikvec_close(n->packs);

	int i;
	for (i = 0; i < n->peermax; ++i)
		if (n->peers[i].state == MIK_CONN)
			mikpeer_close(&n->peers[i]);

	free(n->fds);
	free(n->peers);

	close(n->tcp);
}
