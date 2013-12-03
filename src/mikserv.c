#include <miknet/miknet.h>

/**
 *  Construct a server object, bound to an address and ready for config.
 *
 *  @s: pointer to the server object
 *  @port: port between 0 - 65535
 *  @ip: IPv4/IPv6
 *
 *  @return: 0 on success; negative error code on failure
 */
int mik_serv_make (mikserv_t *s, uint16_t port, mikip_t ip)
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

	s->ip = ip;

	if (ip == MIK_IPV4)
		hint.ai_family = AF_INET;
	else if (ip == MIK_IPV6)
		hint.ai_family = AF_INET6;
	else
		return ERR_INVALID_IP;

	s->tcp = socket(hint.ai_family, SOCK_STREAM, 0);
	s->udp = socket(hint.ai_family, SOCK_DGRAM, 0);
	if (s->udp < 0)
		return mik_debug(ERR_SOCKET);


	hint.ai_flags = AI_PASSIVE;

	hint.ai_socktype = SOCK_STREAM;
	err = getaddrinfo(NULL, portstr, &hint, &serv);
	if (err)
		return mik_debug(ERR_ADDRESS);

	for (p = serv; p; p = p->ai_next) {
		err = bind(s->tcp, p->ai_addr, p->ai_addrlen);
		if (!err) {
			if (MIK_DEBUG)
				mik_print_addr(p->ai_addr, p->ai_addrlen);
			break;
		}
	}

	freeaddrinfo(serv);

	hint.ai_socktype = SOCK_DGRAM;
	err = getaddrinfo(NULL, portstr, &hint, &serv);
	if (err)
		return mik_debug(ERR_ADDRESS);

	for (p = serv; p; p = p->ai_next) {
		err = bind(s->udp, p->ai_addr, p->ai_addrlen);
		if (!err) {
			if (MIK_DEBUG)
				mik_print_addr(p->ai_addr, p->ai_addrlen);
			break;
		}
	}

	freeaddrinfo(serv);

	if (err < 0)
		return mik_debug(ERR_BIND);

	err = setsockopt(s->tcp, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	err = setsockopt(s->udp, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	if (err < 0)
		return mik_debug(ERR_SOCK_OPT);

	err = listen(s->tcp, MIK_WAIT_MAX);
	if (err < 0)
		return mik_debug(ERR_LISTEN);

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
	s->fds = calloc(2 + s->peermax, sizeof(struct pollfd));
	s->peers = calloc(s->peermax, sizeof(mikpeer_t));
	s->fds[0].fd = s->tcp;
	s->fds[1].fd = s->udp;
	s->fds[0].events = POLLIN;
	s->fds[1].events = POLLIN;

	int i;
	for (i = 0; i < s->peermax; ++i)
		s->fds[i + 2].fd = -1;

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

	int err = poll(s->fds, s->peermax + 2, t);

	if (err < 0)
		return mik_debug(ERR_POLL);

	mik_poll(s);

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

	close(s->tcp);
	close(s->udp);

	free(s->peers);
	free(s->fds);

	return 0;
}
