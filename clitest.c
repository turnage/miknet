#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err;
	miknode_t node = {0};

	err = miknode(&node, MIK_IPV6, 7000);
	err = miknode_config(&node, 20, 0, 0);

	return 0;
}
