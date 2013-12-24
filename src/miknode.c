#include <miknet/miknet.h>

static int mik_sock (int *t, struct addrinfo *h)
{
	int err, yes = 1;

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
	int err, bound = 0;
	struct addrinfo *li, *i, c;

	c = *h;
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

	int i, events = 0;
	int err = poll(n->fds, 1 + n->peermax, t);

	/* Connection on master TCP socket. */
	if (n->fds[0].revents & POLLIN) {
		err = mikpeer(n);
		if (err < 0)
			mik_debug(err);
	}

	for (i = 0; i < n->peermax; ++i) {
		if (n->fds[1 + i].revents & POLLIN) {
			mikpeer_recv(&n->peers[i]);
			n->fds[1 + i].revents = 0;
		}
	}

	mikevent_t *event;

	while (n->commands) {
		event = (mikevent_t *)n->commands->data;
		int sock = n->peers[event->peer].tcp;
		void *data = (void *)event->pack.data;
		char buffer[sizeof(mikpack_t) + event->pack.len];

		memset(buffer, 0, sizeof(mikpack_t) + event->pack.len);
		memcpy(buffer, &event->pack, sizeof(mikpack_t));
		memcpy(buffer + sizeof(mikpack_t), data, event->pack.len);

		send(sock, buffer, sizeof(mikpack_t) + event->pack.len, 0);

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
