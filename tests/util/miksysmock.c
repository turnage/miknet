#include "testing/miksysmock.h"

static void mikfreeaddrinfo_mock(posix_t *posmock, struct addrinfo *res) {}

static int mikgetaddrinfo_mock( posix_t *posmock,
				const char *node,
				const char *service,
				const struct addrinfo *hints,
				struct addrinfo **res)
{
	*res = ((posix_mock_t *)posmock)->getaddrinfo_arg_set;
	return ((posix_mock_t *)posmock)->getaddrinfo_return;
}

static int miksetsockopt_mock(	posix_t *posmock,
				int sockfd,
				int level,
				int optname,
				const void *optval,
				socklen_t optlen)
{
	return ((posix_mock_t *)posmock)->setsockopt_return;
}

static int miksocket_mock(posix_t *posmock, int domain, int type, int protocol)
{
	return ((posix_mock_t *)posmock)->socket_return;
}

posix_t mikposixmock()
{
	posix_t mock;

	mock.freeaddrinfo = mikfreeaddrinfo_mock;
	mock.getaddrinfo = mikgetaddrinfo_mock;
	mock.setsockopt = miksetsockopt_mock;
	mock.socket = miksocket_mock;

	return mock;
}
