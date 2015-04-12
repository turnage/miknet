#include "testing/miksysmock.h"

static int mikbind_mock(	const mikposix_t *mock,
				int sockfd,
				const struct sockaddr *addr,
				socklen_t addrlen)
{
	return ((const posix_mock_t *)mock)->bind_return;
}

static void mikfreeaddrinfo_mock(	const mikposix_t *mock,
					struct addrinfo *res) {}

static int mikgetaddrinfo_mock( const mikposix_t *mock,
				const char *node,
				const char *service,
				const struct addrinfo *hints,
				struct addrinfo **res)
{
	*res = ((const posix_mock_t *)mock)->getaddrinfo_arg_set;
	return ((const posix_mock_t *)mock)->getaddrinfo_return;
}

static ssize_t miksendto_mock(	const mikposix_t *mock,
				int sockfd,
				const void *buf,
				size_t len,
				int flags,
				const struct sockaddr *dest_addr,
				socklen_t addrlen)
{
	return ((const posix_mock_t *)mock)->sendto_return;
}

static int miksetsockopt_mock(	const mikposix_t *mock,
				int sockfd,
				int level,
				int optname,
				const void *optval,
				socklen_t optlen)
{
	return ((const posix_mock_t *)mock)->setsockopt_return;
}

static int miksocket_mock(	const mikposix_t *mock,
				int domain,
				int type,
				int protocol)
{
	return ((const posix_mock_t *)mock)->socket_return;
}

static ssize_t mikrecvfrom_mock(	const mikposix_t *mock,
					int sockfd,
					void *buf,
					size_t len,
					int flags,
					struct sockaddr *src_addr,
					socklen_t *addrlen)
{
	return ((const posix_mock_t *)mock)->recvfrom_return;
}

mikposix_t mikposixmock()
{
	mikposix_t mock = {	mikbind_mock,
				mikfreeaddrinfo_mock,
				mikgetaddrinfo_mock,
				miksendto_mock,
				miksetsockopt_mock,
				miksocket_mock,
				mikrecvfrom_mock};

	return mock;
}
