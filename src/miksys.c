#include "miknet/miksys.h"

posix_t mikposix()
{
	posix_t posix = {	freeaddrinfo,
				getaddrinfo,
				setsockopt,
				socket};

	return posix;
}
