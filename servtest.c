#include <miknet/miknet.h>

int main (int argc, char **argv)
{
	mikserv_t server;
	int err, sock;

	err = mik_serv_make(&server, 8015, MIK_SAFE, MIK_IPV6);
	printf("Status: %s\n", mik_errstr(err));

	sock = accept(server.sock, NULL, NULL);
	close(sock);

	mik_serv_close(&server);

	return 0;
}
