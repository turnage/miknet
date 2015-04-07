#include <stdio.h>
#include <stdlib.h>

#include "miknet/mikdef.h"
#include "miknet/mikgram.h"
#include "miknet/miknode.h"

#define FAIL(x) fprintf(stderr, x); exit(0)

int main(int argc, char **argv)
{
	miknode_t *node;
	mikgram_t gram;

	if (argc != 3)
		printf("Usage: ./simple_send [address] [port]\n");

	fprintf(stderr, "Sending mikgram to %s:%s.\n", argv[1], argv[2]);

	node = miknode(0, 1);
	if (node == NULL) {
		FAIL("Failed to initialize miknet.\n");
	} else {
		fprintf(stderr, "Initialized miknet.\n");
	}

	if (miknode_new_peer(node, argv[1], atol(argv[2])) < 0) {
		FAIL("Failed to lookup the given address.\n");
	} else {
		fprintf(stderr, "Added %s:%s as a peer.\n", argv[1], argv[2]);
	}

	if (miknode_send(node, 0, "Hello", 6) != MIKERR_NONE) {
		FAIL("Failed to queue data for sending.\n");
	} else {
		fprintf(stderr, "Data queued to send.\n");
	}

	mikgram_close(&gram);
	miknode_close(node);

	return 0;
}
