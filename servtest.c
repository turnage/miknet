#include <tubuil/tubuil.h>

int main (int argc, char **argv)
{
	tubserv_t server;
	int err, sock;

	err = tub_serv_make(&server, 8015, TUB_SAFE, TUB_IPV6);
	printf("Status: %s\n", tub_errstr(err));

	sock = accept(server.sock, NULL, NULL);
	close(sock);

	tub_serv_close(&server);

	return 0;
}
