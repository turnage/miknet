#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err = 0;

	/* Create a node and initialize it to zero (this is important). */
	miknode_t node = {0};

	/* Create the node and bind it to a port. */
	err = miknode(&node, MIK_IPV4, 7000);
	if (err < 0) {
		fprintf(stderr, "Failed to create node.\n");
		return -1;
	}

	/* Set the peer and bandwidth limits for the node. */
	err = miknode_config(&node, 20, 0, 0);
	if (err < 0){
		fprintf(stderr, "Failed to configure node.\n");
		return -1;
	}

	/* Connect to a peer. This function will return the slot
	   in which the new peer's data is stored, or -1 on failure. */
	int position = mikpeer_connect(&node, argv[1], 8000);
	if (position < 0){
		fprintf(stderr, "Failed to connect node.\n");
		return -1;
	}

	/* Queue some data to be sent to the peer. */
	mikpeer_send(&node.peers[position], "Hello!", 6);

	/* The server example program will take this as a shut down signal. */
	mikpeer_send(&node.peers[position], "quit", 4);

	/* Let the node execute queued commands and collect
	   incoming packets. Provide it a ~maximum blocking
	   time in milliseconds. */
	miknode_poll(&node, 100);

	/* Close the node and free all the memory it holds. */
	miknode_close(&node);

	return 0;
}
