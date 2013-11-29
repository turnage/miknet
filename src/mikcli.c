#include <miknet/miknet.h>

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
		if (!err) {
			mik_print_addr(p->ai_addr, p->ai_addrlen);
			break;
		}
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
