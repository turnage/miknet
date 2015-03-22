#ifndef MIKNET_MIKSYSMOCK_H_
#define MIKNET_MIKSYSMOCK_H_

#include "miknet/miksys.h"

/**
 *  To control the behavior of the mocked function in posix_t, create an
 *  instance of posix_mock_t, initialize the posix field with mikposixmock(),
 *  and fill the fields with the desired behaviors.
 *
 *  This struct can be passed into functions expecting a pointer to a
 *  mikposix_t.
 */
typedef struct posix_mock_t {
	posix_t posix;
	int bind_return;
	int getaddrinfo_return;
	struct addrinfo *getaddrinfo_arg_set;
	ssize_t sendto_return;
	int setsockopt_return;
	int socket_return;
	ssize_t recvfrom_return;
} posix_mock_t;


/**
 *  Returns a posix function wrapper which directs to the mock functions,
 *  instead of the actual ones.
 */
posix_t mikposixmock();

#endif /* MIKNET_MIKSYSMOCK_H_ */
