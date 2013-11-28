#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	mikserv_t server;
	int err;

	err = mik_serv_make(&server, 8016, MIK_SAFE, MIK_IPV6);
	printf("Make:   %s\n", mik_errstr(err));
	err = mik_serv_config(&server, 100, 0, 0);
	printf("Config: %s\n", mik_errstr(err));

	while(server.peerc < 1) {
		mik_serv_poll(&server, 1000);
	}

	mik_serv_close(&server);

	return 0;
}
