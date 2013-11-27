#include <miknet/miknet.h>

static void print_addr (struct sockaddr *a, socklen_t l)
{
	char hostname[256] = {0};
	char service[256] = {0};
	getnameinfo(a, l, hostname, 256, service, 256, 0);
	fprintf(stderr, "Bound to: %s:%s.\n", hostname, service);
}

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
	if (s->sock < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_SOCKET;
	}

	hint.ai_flags = AI_PASSIVE;

	err = getaddrinfo(NULL, portstr, &hint, &serv);
	if (err) {
		if (MIK_DEBUG)
			fprintf(stderr, "Net err: %s.\n", gai_strerror(err));
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
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_BIND;
	}

	err = setsockopt(s->sock, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	if (err < 0) {
		if (MIK_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_SOCK_OPT;
	}

	if ((mode == MIK_TCP) || (mode == MIK_SAFE)) {
		err = listen(s->sock, MIK_WAIT_MAX);
		if (err < 0) {
			if (MIK_DEBUG)
				fprintf(stderr, "Net err: %s.\n", strerror(errno));
		}
	}

	return 0;
}

int mik_serv_close (mikserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	close(s->sock);

	return 0;
}

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

int mik_cli_close (mikcli_t *c)
{
	if (!c)
		return ERR_MISSING_PTR;

	close(c->sock);

	return 0;
}
