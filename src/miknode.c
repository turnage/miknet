#include <miknet/miknet.h>

static int mik_sockpair (int *t, int *u, struct addrinfo *h)
{
	int err, yes = 1;

	*t = socket(h->ai_family, SOCK_STREAM, 0);
	*u = socket(h->ai_family, SOCK_DGRAM, 0);
	if ((*t < 0) || (*u < 0))
		return mik_debug(ERR_SOCKET);

	err = setsockopt(*t, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	err += setsockopt(*u, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
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
			h->ai_addr = i->ai_addr;
			h->ai_addrlen = i->ai_addrlen;
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
 *  @u: SOCK_DGRAM socket
 *  @h: copy of address request
 *  @p: port or 0 for auto-assign
 *
 *  @return: the port bound to
 */
int mik_bind (int *t, int *u, struct addrinfo h, uint16_t p)
{
	int socks = 0;
	uint16_t port;
	char portstr[MIK_PORT_MAX] = {0};

	if (p == 0)
		port = 1025;
	else
		port = p;

	mik_sockpair(t, u, &h);

	while (socks < 2) {
		memset(portstr, 0, MIK_PORT_MAX);
		sprintf(portstr, "%u", port);
		h.ai_addr = 0;
		h.ai_addrlen = 0;

		h.ai_socktype = SOCK_STREAM;
		if (mik_testbind(*t, &h, portstr) > 0)
			socks += 1;

		h.ai_socktype = SOCK_DGRAM;
		if (mik_testbind(*u, &h, portstr) > 0)
			socks += 1;

		if (socks < 2) {
			close(*t);
			close(*u);
			mik_sockpair(t, u, &h);
			socks = 0;
			port++;
		}
	}

	return port;
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

	mik_bind(&n->tcp, &n->udp, hint, port);

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
	n->fds = calloc(n->peermax + 2, sizeof(mikpeer_t));
	if (!n->peers || !n->fds)
		return mik_debug(ERR_MEMORY);

	n->fds[0].fd = n->tcp;
	n->fds[1].fd = n->udp;
	n->fds[0].events = POLLIN;
	n->fds[1].events = POLLIN;

	memset(n->packs, 0, sizeof(n->packs));

	return 0;
}