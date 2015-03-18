#include "testing/miksysmock.h"

static int mikbind_mock(	const posix_t *mock,
				int sockfd,
				const struct sockaddr *addr,
				socklen_t addrlen)
{
	return ((const posix_mock_t *)mock)->bind_return;
}

static void mikfreeaddrinfo_mock(	const posix_t *mock,
					struct addrinfo *res) {}

static int mikgetaddrinfo_mock( const posix_t *mock,
				const char *node,
				const char *service,
				const struct addrinfo *hints,
				struct addrinfo **res)
{
	*res = ((const posix_mock_t *)mock)->getaddrinfo_arg_set;
	return ((const posix_mock_t *)mock)->getaddrinfo_return;
}

static ssize_t miksend_mock(	const posix_t *mock,
				int sockfd,
				const void *buf,
				size_t len,
				int flags)
{
	return ((const posix_mock_t *)mock)->send_return;
}

static int miksetsockopt_mock(	const posix_t *mock,
				int sockfd,
				int level,
				int optname,
				const void *optval,
				socklen_t optlen)
{
	return ((const posix_mock_t *)mock)->setsockopt_return;
}

static int miksocket_mock(	const posix_t *mock,
				int domain,
				int type,
				int protocol)
{
	return ((const posix_mock_t *)mock)->socket_return;
}

static ssize_t mikrecv_mock(	const posix_t *mock,
				int sockfd,
				void *buf,
				size_t len,
				int flags)
{
	return ((const posix_mock_t *)mock)->recv_return;
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
