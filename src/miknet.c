#include <miknet/miknet.h>

uint32_t MIK_TCP_MAX = MIK_PACK_MAX;

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

void *try_alloc(void *ptr, size_t bytes)
{
	void *ret = realloc(ptr, bytes);

	if (!ret && MIK_DEBUG)
		fprintf(stderr, "Memory failure; ptr: %p.\n", ptr);

	return ret ? ret : ptr;
}

/**
 *  Set the TCP read size (for MIK_BARE peers).
 *
 *  @size: the new read size
 */
void mik_set_readsize (uint32_t size)
{
	MIK_TCP_MAX = size;
}

/**
 *  Interpret metadata from a character array (for receiving packets).
 *
 *  @meta: pointer to the data read of the packet header
 *
 *  @return: a mikmeta_t object built from the header
 */
mikmeta_t mik_read_meta (char *meta)
{
	mikmeta_t data = {0};

	if (!meta)
		return data;

	memcpy(&data.channel, meta, MIK_CHAN_SZ);
	memcpy(&data.len, meta + MIK_CHAN_SZ, MIK_LEN_SZ);

	data.channel = ntohl(data.channel);
	data.len = ntohs(data.len);

	return data;
}

/**
 *  Encode a packet header for sending safely over the network.
 *
 *  @data: the data to be written
 *  @meta: a pointer to a char array of at least length MIK_META_SZ
 *
 *  @return: 0 on success; and error code less than 0 otherwise
 */
int mik_write_meta (mikpack_t data, char *meta)
{
	if (!meta)
		return ERR_MISSING_PTR;

	data.channel = htonl(data.channel);
	data.len = htons(data.len);

	memcpy(meta, &data.channel, MIK_CHAN_SZ);
	memcpy(meta + MIK_CHAN_SZ, &data.len, MIK_LEN_SZ);

	return 0;
}

/**
 *  Fetch the next event for the programmer to handle.
 *
 *  @node: pointer to the node
 *
 *  @return: pointer to the next event or NULL
 */
mikpack_t *mikevent (miknode_t *node)
{
	if (!node)
		return NULL;

	mikpack_t *event = mikvec_next(&node->packs);

	if (!event)
		node->packs = mikvec_clear(node->packs);

	return event;
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
