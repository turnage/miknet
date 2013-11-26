#include <tubuil/tubuil.h>

int main (int argc, char **argv)
{
	tubcli_t client;
	int err;

	err = tub_cli_make(&client, TUB_SAFE, TUB_IPV4);
	printf("Status: %s\n", tub_errstr(err));

	err = tub_cli_connect(&client, 8015, "127.0.0.1");
	printf("Status: %s\n", tub_errstr(err));

	tub_cli_close(&client);

	return 0;
}
