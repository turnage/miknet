#include <miknet/miknet.h>

static void print_addr (struct sockaddr *a, socklen_t l)
{
	char hostname[256] = {0};
	char service[256] = {0};
	getnameinfo(a, l, hostname, 256, service, 256, 0);
	fprintf(stderr, "Bound to: %s:%s.\n", hostname, service);
}

static int tcp_peer (mikserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	int err;

	struct sockaddr_storage a;
	socklen_t alen;

	err = accept(s->sock, (struct sockaddr *)&a, &alen);
	if (err < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "SYS: %s.\n", strerror(errno));
		return ERR_SOCKET;
	}
	s->peerc++;
	s->peers = realloc(s->peers, s->peerc * sizeof(mikpeer_t));
	memset(s->peers + s->peerc - 1, 0, sizeof(mikpeer_t));
	if (!s->peers)
		return ERR_MEMORY;
	mikpeer_t *p;
	for (p = s->peers; p; p = p->next);
	p->prev = (s->peerc > 1) ? p - 1 : NULL;
	p->sock = err;
	p->addr = a;
	p->addrlen = alen;

	if (s->ip == MIK_IPV4) {
		struct sockaddr_in *a4 = (struct sockaddr_in *)&p->addr;
		inet_ntop(p->addr.ss_family,
			&a4->sin_addr, p->ip, MIK_IPST_MAX);
	} else if (s->ip == MIK_IPV6) {
		struct sockaddr_in6 *a6 = (struct sockaddr_in6 *)&p->addr;
		inet_ntop(p->addr.ss_family,
			&a6->sin6_addr, p->ip, MIK_IPST_MAX);
	}
	
	if (MIK_DEBUG) {
		fprintf(stderr, "Client [%d]: %s.\n", s->peerc, p->ip);
	}

	return 0;
}

static int tcp_poll (mikserv_t *s, int t)
{
	if (!s)
		return ERR_MISSING_PTR;

	int err;

	if ((s->mode != MIK_TCP) && (s->mode != MIK_SAFE))
		return ERR_INVALID_MODE;

	err = poll(s->fds, s->nfds, t);
	if (err < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "SYS: %s.\n", strerror(errno));
		return ERR_POLL;
	}
	if (s->fds->revents & POLLIN) {
		return tcp_peer(s);
	}

	return 0;
}

/**
 *  Convert an error code into a human-readable string.
 *
 *  @err: error code
 *
 *  @return: pointer to the string
 */
const char *mik_errstr(int err)
{
	const char *str;

	switch (err) {
		case 0:	{
			str = "No errors detected.";
			break;
		}

		case ERR_MISSING_PTR: {
			str = "A passed pointer was NULL.";
			break;
		}

		case ERR_INVALID_MODE: {
			str = "Network mode invalid.";
			break;
		}

		case ERR_INVALID_IP: {
			str = "IP address type invalid.";
			break;
		}

		case ERR_SOCKET: {
			str = "Failed to create socket.";
			break;
		}

		case ERR_ADDRESS: {
			str = "Address invalid or taken.";
			break;
		}

		case ERR_SOCK_OPT: {
			str = "Failed to set socket options.";
			break;
		}

		case ERR_BIND: {
			str = "Failed to bind socket.";
			break;
		}

		case ERR_CONNECT: {
			str = "Failed to connect socket.";
			break;
		}

		default: str = "Unrecognized error code.";
	}

	return str;
}

/**
 *  Construct a server object, bound to an address and ready for config.
 *
 *  @s: pointer to the server object
 *  @port: port between 0 - 65535
 *  @mode: TCP/UDP; SAFE/FAST
 *  @ip: IPv4/IPv6
 *
 *  @return: 0 on success; negative error code on failure
 */
int mik_serv_make (mikserv_t *s, uint16_t port, miknet_t mode, mikip_t ip)
{
	if (!s)
		return ERR_MISSING_PTR;

	struct addrinfo hint, *serv, *p;
	char portstr[MIK_PORT_MAX] = {0};
	int yes = 1;
	int err;

	memset(s, 0, sizeof(mikserv_t));
	memset(&hint, 0, sizeof(hint));
	sprintf(portstr, "%d", port);

	if ((mode == MIK_UDP) || (mode == MIK_FAST)) {
		hint.ai_socktype = SOCK_DGRAM;
	} else if ((mode == MIK_TCP) || (mode == MIK_SAFE)) {
		hint.ai_socktype = SOCK_STREAM;
	} else
		return ERR_INVALID_MODE;

	if (ip == MIK_IPV4)
		hint.ai_family = AF_INET;
	else if (ip == MIK_IPV6)
		hint.ai_family = AF_INET6;
	else
		return ERR_INVALID_IP;

	s->sock = socket(hint.ai_family, hint.ai_socktype, 0);
	printf("Socket fd: %d.\n", s->sock);
	if (s->sock < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "SYS: %s.\n", strerror(errno));
		return ERR_SOCKET;
	}

	hint.ai_flags = AI_PASSIVE;

	err = getaddrinfo(NULL, portstr, &hint, &serv);
	if (err) {
		if (MIK_DEBUG)
			fprintf(stderr, "SYS: %s.\n", gai_strerror(err));
		return ERR_ADDRESS;
	}

	for (p = serv; p; p = p->ai_next) {
		err = bind(s->sock, p->ai_addr, p->ai_addrlen);
		if (!err) {
			if (MIK_DEBUG)
				print_addr(p->ai_addr, p->ai_addrlen);
			break;
		}
	}

	freeaddrinfo(serv);

	if (err < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "SYS: %s.\n", strerror(errno));
		return ERR_BIND;
	}

	err = setsockopt(s->sock, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	if (err < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "SYS: %s.\n", strerror(errno));
		return ERR_SOCK_OPT;
	}

	if ((mode == MIK_TCP) || (mode == MIK_SAFE)) {
		err = listen(s->sock, MIK_WAIT_MAX);
		if (err < 0) {
			if (MIK_DEBUG)
				fprintf(stderr, "SYS: %s.\n", strerror(errno));
		}
	}

	return 0;
}

/**
 *  Configure a server for operation.
 *
 *  @s: pointer to the server object
 *  @pc: maximum peers
 *  @u: maximum upbandwidth/s
 *  @d: maximum downbandwidth/s
 *
 *  @return: 0 on success; negative error code on failure
 */
int mik_serv_config (mikserv_t *s, uint16_t pc, uint32_t u, uint32_t d)
{
	if (!s)
		return ERR_MISSING_PTR;

	if (pc > MIK_PEER_MAX)
		return ERR_PEER_MAX;

	s->peerc = pc;
	s->upcap = u;
	s->downcap = d;

	/* Note: in UDP mode, this is the only pollfd. */
	s->fds = calloc(1, sizeof(struct pollfd));
	s->nfds = 1;
	s->fds->fd = s->sock;
	s->fds->events = POLLIN;

	return 0;
}

/**
 *  Queue received data for processing and dequeue packets waiting to be sent.
 *
 *  @s: pointer to the server object
 *  @t: target blocking time in milliseconds
 *
 *  @return: 0 on success, negative error code on failure
 */
int mik_serv_poll (mikserv_t *s, int t)
{
	if (!s)
		return ERR_MISSING_PTR;

	if ((s->mode == MIK_TCP) || (s->mode == MIK_SAFE))
		return tcp_poll(s, t);

	/* TODO: UDP monitor. */

	return 0;
}

/**
 *  Release all the resources held by a server object.
 *
 *  @s: pointer to server object
 *
 *  @return: 0 on success; negative error code on failure
 */
int mik_serv_close (mikserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	close(s->sock);

	free(s->peers);
	free(s->fds);

	return 0;
}

/**
 *  Create a client object, which is ready to connect somewhere.
 *
 *  @c: pointer to client object
 *  @mode: TCP/UDP; SAFE/FAST
 *  @ip: IPv4/IPv6
 *
 *  @return: 0 on success; negative error code on failure
 */
int mik_cli_make (mikcli_t *c, miknet_t mode, mikip_t ip)
{
	if (!c)
		return ERR_MISSING_PTR;

	memset(c, 0, sizeof(mikcli_t));

	if ((mode == MIK_UDP) || (mode == MIK_FAST)) {
		c->meta.ai_socktype = SOCK_DGRAM;
	} else if ((mode == MIK_TCP) || (mode == MIK_SAFE)) {
		c->meta.ai_socktype = SOCK_STREAM;
	} else
		return ERR_INVALID_MODE;

	if (ip == MIK_IPV4)
		c->meta.ai_family = AF_INET;
	else if (ip == MIK_IPV6)
		c->meta.ai_family = AF_INET6;
	else
		return ERR_INVALID_IP;

	c->sock = socket(c->meta.ai_family, c->meta.ai_socktype, 0);
	if (c->sock < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_SOCKET;
	}

	return 0;
}


/**
 *  Connect a client to a server (not necessarily a miknet server).
 *
 *  @c: pointer to client object
 *  @port: port between 0 - 65535
 *  @addr: hostname or NULL for localhost
 *  
 *  @return: 0 on success; negative error code on failure
 */
int mik_cli_connect (mikcli_t *c, uint16_t port, const char *addr)
{
	if (!c)
		return ERR_MISSING_PTR;

	if (!addr)
		c->meta.ai_flags = AI_PASSIVE;

	int err;
	struct addrinfo *serv, *p;
	char portstr[MIK_PORT_MAX] = {0};

	sprintf(portstr, "%d", port);

	err = getaddrinfo(addr, portstr, &c->meta, &serv);
	if (err) {
		if (MIK_DEBUG)
			fprintf(stderr, "Net err: %s.\n", gai_strerror(err));
		return ERR_ADDRESS;
	}

	for (p = serv; p; p = p->ai_next) {
		err = connect(c->sock, p->ai_addr, p->ai_addrlen);
		if (!err)
			break;
	}

	freeaddrinfo(serv);

	if (err < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_CONNECT;
	}

	return 0;
}

/**
 *  Release all resourced held by the client object.
 *
 *  @c: pointer to client object
 *
 *  @return: 0 on success; negative error code on failure
 */
int mik_cli_close (mikcli_t *c)
{
	if (!c)
		return ERR_MISSING_PTR;

	close(c->sock);

	return 0;
}
