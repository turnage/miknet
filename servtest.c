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
		   incoming event->. Provide it a ~maximum blocking
		   time in milliseconds. */
		miknode_poll(&node, 100);

		/* After a call to miknode_poll, fetch events from mikevent
                   until there are no more events to handle. Make sure to
                   handle all events before polling again, or those events will
                   be lost. */

		mikpack_t *event = mikevent(&node);

		while (event) {

			/* Event types
				MIK_JOIN: A new peer joined; data field is NULL.
				MIK_QUIT: A peer quit; data field is NULL.
				MIK_DATA: A peer sent data; data field is set. 
			*/

			/* Miknet event-> can be sent over virtual channels,
			   market by an unsigned 32 bit integer. The default
			   channel (for join/quit notifications) is 0. */
			printf("Packet on channel %u; ", event->channel);

			if (event->type == MIK_JOIN) {
				printf("New peer in slot %d.\n", event->peer);
			} else if (event->type == MIK_QUIT) {
				printf("Lost peer in slot %d.\n", event->peer);
			} else if (event->type == MIK_DATA) {
				printf("Data from peer: %s\n", event->data);

				if (!strncmp(event->data, "quit\0", 5))
					quit = 1;
			}

			event = mikevent(&node);
		}
	}

	/* Close the node and free all the memory it holds. */
	miknode_close(&node);

	return 0;
}
