#include <miknet/miknet.h>

void mik_print_addr (struct sockaddr *a, socklen_t l)
{
	char hostname[256] = {0};
	char service[256] = {0};
	getnameinfo(a, l, hostname, 256, service, 256, 0);
	fprintf(stderr, "Bound to: %s:%s.\n", hostname, service);
}

int mik_tcp_peer (mikserv_t *s)
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

	if (s->peerc == s->peermax) {
		close(err);
		return ERR_PEER_MAX;
	}

	s->peerc++;	
	s->peers = realloc(s->peers, s->peerc * sizeof(mikpeer_t));
	s->fds = realloc(s->fds, s->nfds * sizeof(struct pollfd));
	if (!s->peers || !s->fds)
		return ERR_MEMORY;

	memset(&s->peers[s->peerc - 1], 0, sizeof(mikpeer_t));
	s->peers[s->peerc - 1].sock = err;
	s->peers[s->peerc - 1].addr = a;
	s->peers[s->peerc - 1].addrlen = alen;
	if (s->peerc > 1) {
		s->peers[s->peerc - 2].next = &s->peers[s->peerc - 1];
		s->peers[s->peerc - 1].prev = &s->peers[s->peerc - 2];
	}

	memset(&s->fds[s->nfds - 1], 0, sizeof(struct pollfd));
	s->fds[s->nfds - 1].fd = s->peers[s->peerc - 1].sock;
	s->fds[s->nfds - 1].events = POLLIN;

	if (MIK_DEBUG) {
		fprintf(stderr, "Client [%d]: %s.\n", s->peerc,
			s->peers[s->peerc - 1].ipst);
	}

	return 0;
}

int mik_tcp_poll (mikserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	if (s->fds[0].revents & POLLIN) {
		return mik_tcp_peer(s);
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
