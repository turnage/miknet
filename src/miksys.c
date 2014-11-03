#include "miknet/miksys.h"

posix_t mikposix()
{
	posix_t posix = {	connect,
				freeaddrinfo,
				getaddrinfo,
				setsockopt,
				socket};

	return posix;
}
