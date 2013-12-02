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
int mik_cli_make (mikcli_t *c, mikip_t ip)
{
	if (!c)
		return ERR_MISSING_PTR;

	struct addrinfo meta;

	memset(&meta, 0, sizeof(struct addrinfo));
	memset(c, 0, sizeof(mikcli_t));


	if (ip == MIK_IPV4)
		meta.ai_family = AF_INET;
	else if (ip == MIK_IPV6)
		meta.ai_family = AF_INET6;
	else
		return ERR_INVALID_IP;

	c->tcp = socket(meta.ai_family, SOCK_STREAM, 0);
	c->udp = socket(meta.ai_family, SOCK_DGRAM, 0);
	if (c->udp < 0)
		return mik_debug(ERR_SOCKET);

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

	int err;
	struct addrinfo meta, *serv, *p;
	char portstr[MIK_PORT_MAX] = {0};

	if (!addr)
		meta.ai_flags = AI_PASSIVE;

	memset(&meta, 0, sizeof(struct addrinfo));
	sprintf(portstr, "%d", port);

	err = getaddrinfo(addr, portstr, &meta, &serv);
	if (err)
		return mik_debug(ERR_ADDRESS);

	for (p = serv; p; p = p->ai_next) {
		err = connect(c->tcp, p->ai_addr, p->ai_addrlen);
		if (!err) {
			mik_print_addr(p->ai_addr, p->ai_addrlen);
			break;
		}
	}

	freeaddrinfo(serv);

	if (err < 0)
		return mik_debug(ERR_CONNECT);

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

	close(c->tcp);
	close(c->udp);

	return 0;
}
