#include <miknet/miknet.h>

int mik_debug (int err)
{
	if (MIK_DEBUG) {
		fprintf(stderr, "SYS: %s\n", strerror(errno));
	}

	return err;
}

void mik_print_addr (struct sockaddr *a, socklen_t l)
{
	char hostname[256] = {0};
	char service[256] = {0};
	getnameinfo(a, l, hostname, 256, service, 256, 0);
	fprintf(stderr, "Bound to: %s:%s.\n", hostname, service);
}

int mik_send (int sockfd, miktype_t t, char *data, int len)
{
	if (!data && len)
		return ERR_MISSING_PTR;

	if ((len > MIK_PACK_MAX) || (len < 0))
		return ERR_WOULD_FAULT;

	mikpack_t *p;
	char buffer[sizeof(mikpack_t) + len];

	p = (mikpack_t *)buffer;
	memset(buffer, 0, sizeof(mikpack_t) + len);

	p->meta = t;
	p->len = len;

	if (data)
		memcpy(buffer + sizeof(mikpack_t), data, len);
	
	int err = send(sockfd, buffer, sizeof(mikpack_t) + len, 0);
	if (err < 0)
		return mik_debug(ERR_SOCKET);

	return 0;
}

mikpack_t mik_tcp_recv (int sockfd, uint16_t peer)
{
	mikpack_t p, *t;
	int len = sizeof(mikpack_t) + MIK_PACK_MAX;
	char buffer[len];

	t = (mikpack_t *)buffer;
	memset(&p, 0, sizeof(mikpack_t));
	memset(buffer, 0, len);

	int err = recv(sockfd,	buffer, len, 0);
	if (err < 0) {
		mik_debug(ERR_SOCKET);
		return p;
	}

	if (t->len > MIK_PACK_MAX) {
		mik_debug(ERR_WOULD_FAULT);
		return p;
	}

	p.meta = t->meta;
	p.len = t->len;
	p.peer = peer;
	p.data = calloc(1, t->len);
	if (!p.data) {
		mik_debug(ERR_MEMORY);
		return p;
	}

	memcpy(p.data, buffer + sizeof(mikpack_t), p.len);

	return p;
}

int mik_peer (mikserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	int err, i;

	struct sockaddr_storage a;
	socklen_t alen;

	err = accept(s->tcp, (struct sockaddr *)&a, &alen);
	if (err < 0)
		return mik_debug(ERR_SOCKET);

	if (s->peerc == s->peermax) {
		close(err);
		return ERR_PEER_MAX;
	}

	for (i = 0; i < s->peermax; ++i)
		if (s->peers[i].state == MIK_DISC)
			break;

	memset(&s->peers[i], 0, sizeof(mikpeer_t));
	s->peers[i].tcp = err;
	s->peers[i].addr = a;
	s->peers[i].addrlen = alen;
	s->peerc++;

	memset(&s->fds[i + 2], 0, sizeof(struct pollfd));
	s->fds[i + 2].fd = s->peers[i].tcp;
	s->fds[i + 2].events = POLLIN;

	if (MIK_DEBUG) {
		fprintf(stderr, "Client [%d]: %s.\n", i,
			s->peers[i].ipst);
	}

	return 0;
}

int mik_poll (mikserv_t *s)
{
	if (!s)
		return ERR_MISSING_PTR;

	if (s->fds[0].revents & POLLIN) {
		return mik_peer(s);
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

		case ERR_PEER_MAX: {
			str = "Argument exceeds peer maximum.";
			break;
		}

		case ERR_POLL: {
			str = "The poll call failed.";
			break;
		}

		case ERR_MEMORY: {
			str = "Memory allocation failed.";
			break;
		}

		case ERR_WOULD_FAULT: {
			str = "This operation would have segfaulted.";
			break;
		}

		case ERR_LISTEN: {
			str = "Configuring the SOCK_STREAM backlog failed.";
			break;
		}

		default: str = "Unrecognized error code.";
	}

	return str;
}
