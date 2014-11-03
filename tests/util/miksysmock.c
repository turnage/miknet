#include "testing/miksysmock.h"

static int mock_return = 0;
static uint64_t mock_arg = 0;

static void mikfreeaddrinfo_mock(struct addrinfo *res) {}

static int mikgetaddrinfo_mock(	const char *node,
				const char *service,
				const struct addrinfo *hints,
				struct addrinfo **res)
{
	*res = (struct addrinfo *)mock_arg;
	return mock_return;
}

static int miksetsockopt_mock(	int sockfd,
				int level,
				int optname,
				const void *optval,
				socklen_t optlen)
{
	return mock_return;
}

static int miksocket_mock(int domain, int type, int protocol)
{
	return mock_return;
}

void miksysmock_init()
{
	syswrapper_t wrapper;

	wrapper.freeaddrinfo = mikfreeaddrinfo_mock;
	wrapper.getaddrinfo = mikgetaddrinfo_mock;
	wrapper.setsockopt = miksetsockopt_mock;
	wrapper.socket = miksocket_mock;

	miksys_remap(wrapper);
}

void miksysmock_set_return(int value) { mock_return = value; }

void miksysmock_set_arg(uint64_t value) { mock_arg = value; }
