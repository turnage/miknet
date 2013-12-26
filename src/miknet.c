#include <miknet/miknet.h>

/**
 *  Call this when something goes wrong and you need to know why without making
 *  things ugly.
 */
int mik_debug (int err)
{
	if (MIK_DEBUG) {
		fprintf(stderr, "SYS: %s\n", strerror(errno));
	}

	return err;
}

/**
 *  This is a function for debugging (both while writing the library and with
 *  the library). It will represent an address in presentation form.
 *
 *  @a: the address, likely a casted struct sockaddr_storage pointer
 *  @s: length of the memory a points to
 *
 *  @return: the port number
 */
int mik_print_addr (struct sockaddr *a, socklen_t s)
{
	char hostname[256] = {0};
	char service[256] = {0};
	getnameinfo(a, s, hostname, 256, service, 256, 0);
	fprintf(stderr, "Bound to: %s:%s.\n", hostname, service);

	return atoi(service);
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
	const char *str = NULL;

	switch (err) {
		case 0:	{
			str = "No errors detected.";
			break;
		}

		case ERR_MISSING_PTR: {
			str = "A passed pointer was NULL.";
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