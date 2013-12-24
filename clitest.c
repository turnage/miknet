#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err;
	miknode_t node = {0};

	err = miknode(&node, MIK_IPV4, 7000);
	err = miknode_config(&node, 20, 0, 0);

	fprintf(stderr, "Status: %d.\n", err);

	err = mikpeer_connect(&node, "localhost", 8000);

	fprintf(stderr, "Connected on slot %d.\n", err);

	miknode_close(&node);

	return 0;
}
