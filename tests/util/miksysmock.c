#include "testing/miksysmock.h"

static int mikbind_mock(	posix_t *posmock,
				int sockfd,
				const struct sockaddr *addr,
				socklen_t addrlen)
{
	return ((posix_mock_t *)posmock)->bind_return;
}

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

static ssize_t miksend_mock(	posix_t *posmock,
				int sockfd,
				const void *buf,
				size_t len,
				int flags)
{
	return ((posix_mock_t *)posmock)->send_return;
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

static ssize_t mikrecv_mock(	posix_t *posmock,
				int sockfd,
				void *buf,
				size_t len,
				int flags)
{
	return ((posix_mock_t *)posmock)->recv_return;
}

posix_t mikposixmock()
{
	posix_t mock = {	mikbind_mock,
				mikfreeaddrinfo_mock,
				mikgetaddrinfo_mock,
				miksend_mock,
				miksetsockopt_mock,
				miksocket_mock,
				mikrecv_mock};

	return mock;
}
