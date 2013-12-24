#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err;
	miknode_t node = {0};

	err = miknode(&node, MIK_IPV4, 7000);
	err = miknode_config(&node, 20, 0, 0);

	fprintf(stderr, "Status: %d.\n", err);

	err = mikpeer_connect(&node, "localhost", 8000);
	mikpeer_send(&node.peers[0], MIK_DATA, "Hello", 5);
	miknode_poll(&node, 1000);

	miknode_close(&node);

	return 0;
}
