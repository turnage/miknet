#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	int err;
	miknode_t node = {0};

	err = miknode(&node, MIK_IPV6, 7000);

	return 0;
}
