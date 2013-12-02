#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	mikcli_t client;
	int err;

	err = mik_cli_make(&client, MIK_IPV6);
	printf("Status: %s\n", mik_errstr(err));

	err = mik_cli_connect(&client, 8016, NULL);
	printf("Status: %s\n", mik_errstr(err));

	mik_cli_close(&client);

	return 0;
}
