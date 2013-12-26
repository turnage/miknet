#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err = 0;
	int quit = 0;

	/* Create a node and initialize it to zero (this is important). */
	miknode_t node = {0};

	/* Create the node and bind it to a port. */
	err = miknode(&node, MIK_IPV4, 8000);
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

	while (!quit) {

		/* Let the node execute queued commands and collect
		   incoming packets. Provide it a ~maximum blocking
		   time in milliseconds. */
		miknode_poll(&node, 100);

		/* After each call, node.packs will be a linked list
		   of events that came in during the poll. */
		while (node.packs) {

			mikpack_t packet = node.packs->pack;
			
			/* Packet types
				MIK_JOIN: A new peer joined; data field is NULL.
				MIK_QUIT: A peer quit; data field is NULL.
				MIK_DATA: A peer sent data; data field is set. 
			*/

			if (packet.type == MIK_JOIN) {
				printf("New peer in slot %d.\n", packet.peer);
			} else if (packet.type == MIK_QUIT) {
				printf("Lost peer in slot %d.\n", packet.peer);
			} else if (packet.type == MIK_DATA) {
				printf("Data from peer: %s\n", packet.data);

				if (!strncmp(packet.data, "quit\0", 5))
					quit = 1;
			}

			/* Use miklist_next() to step forward in the packs
			   linked list; it will properly handle freeing the
			   memory used by the packets. */
			node.packs = miklist_next(node.packs);
		}
	}

	/* Close the node and free all the memory it holds. */
	miknode_close(&node);

	return 0;
}