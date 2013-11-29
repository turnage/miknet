#include <miknet/miknet.h>

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

	s->mode = mode;
	s->ip = ip;

	if ((mode == MIK_UDP) || (mode == MIK_FAST))
		hint.ai_socktype = SOCK_DGRAM;
	else if ((mode == MIK_TCP) || (mode == MIK_SAFE))
		hint.ai_socktype = SOCK_STREAM;
	else
		return ERR_INVALID_MODE;

	if (ip == MIK_IPV4)
		hint.ai_family = AF_INET;
	else if (ip == MIK_IPV6)
		hint.ai_family = AF_INET6;
	else
		return ERR_INVALID_IP;

	s->sock = socket(hint.ai_family, hint.ai_socktype, 0);
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
				mik_print_addr(p->ai_addr, p->ai_addrlen);
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
int mik_serv_config (mikserv_t *s, uint16_t pm, uint32_t u, uint32_t d)
{
	if (!s)
		return ERR_MISSING_PTR;

	if (pm > MIK_PEER_MAX)
		return ERR_PEER_MAX;

	s->peermax = pm;
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
 *  Queue received data for processing and dequeue data waiting to be sent.
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
		return mik_tcp_poll(s, t);


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
