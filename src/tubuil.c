#include <tubuil/tubuil.h>

int tub_serv_make (tubserv_t *s, uint16_t port, tubnet_t mode, tubip_t ip)
{
	if (!s)
		return ERR_MISSING_PTR;

	struct addrinfo hint, *serv, *p;
	char portstr[TUB_PORT_MAX] = {0};
	int yes = 1;
	int err;

	memset(s, 0, sizeof(tubserv_t));
	memset(&hint, 0, sizeof(hint));
	sprintf(portstr, "%d", port);

	if (mode == TUB_UDP) {
		hint.ai_socktype = SOCK_DGRAM;
	} else if (mode == TUB_TCP) {
		hint.ai_socktype = SOCK_STREAM;
	} else
		return ERR_INVALID_MODE;

	if (ip == TUB_IPV4)
		hint.ai_family = AF_INET;
	else if (ip == TUB_IPV6)
		hint.ai_family = AF_INET6;
	else
		return ERR_INVALID_IP;

	s->sock = socket(hint.ai_family, hint.ai_socktype, 0);
	if (s->sock < 0) {
		if (TUB_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_SOCKET;
	}

	hint.ai_flags = AI_PASSIVE;

	err = getaddrinfo(NULL, portstr, &hint, &serv);
	if (err) {
		if (TUB_DEBUG)
			fprintf(stderr, "Net err: %s.\n", gai_strerror(err));
		return ERR_ADDRESS;
	}

	for (p = serv; p; p = p->ai_next) {
		err = bind(s->sock, p->ai_addr, p->ai_addrlen);
		if (!err)
			break;
	}

	freeaddrinfo(serv);

	if (err < 0) {
		if (TUB_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_BIND;
	}

	err = setsockopt(s->sock, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int));
	if (err < 0) {
		if (TUB_DEBUG)
			fprintf(stderr, "Net err: %s.\n", strerror(errno));
		return ERR_SOCK_OPT;
	}

	listen(s->sock, TUB_WAIT_MAX);

	return 0;
}

int tub_serv_close (tubserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	close(s->sock);

	return 0;
}