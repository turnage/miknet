#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err, i;
	miknode_t node = {0};

	err = miknode(&node, MIK_IPV4, 8000);
	err = miknode_config(&node, 20, 0, 0);

	for (i = 0; i < 20; ++i) {
		miknode_poll(&node, 1000);
	}

	miknode_close(&node);

	return 0;
}
