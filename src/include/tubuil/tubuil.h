#ifndef tubuil_h
#define tubuil_h

#include <stdio.h>
#include <errno.h>
#include <stdint.h>
#include <string.h>

#include <netdb.h>
#include <unistd.h>
#include <netinet/in.h>

#include <sys/socket.h>
#include <sys/types.h>
#include <sys/time.h>

#define TUB_PEER_MAX 100
#define TUB_CHAN_MAX 100
#define TUB_PACK_MAX 1200
#define TUB_PORT_MAX 6
#define TUB_WAIT_MAX 64

#define TUB_DEBUG 1

enum {
	ERR_MISSING_PTR  = -1,
	ERR_INVALID_MODE = -2,
	ERR_INVALID_IP   = -3,
	ERR_SOCKET       = -4,
	ERR_ADDRESS      = -5,
	ERR_SOCK_OPT     = -6,
	ERR_BIND         = -7,
	ERR_CONNECT      = -8
};

typedef enum {
	TUB_FAST = 1,
	TUB_UDP  = 1,
	TUB_SAFE = 2,
	TUB_TCP  = 3
} tubnet_t;

typedef enum {
	TUB_IPV4 = 1,
	TUB_IPV6 = 2
} tubip_t;

typedef struct tubpeer_t {
	int sock;
	struct tubpeer_t *prev;
	struct tubpeer_t *next;
} tubpeer_t;

typedef struct tubpack_t {
	uint16_t len;
	char data[TUB_PACK_MAX];
} tubpack_t;

typedef struct tubserv_t {
	int sock;
	struct sockaddr address;
	tubnet_t socktype;
	tubip_t iptype;
} tubserv_t;

typedef struct tubcli_t {
	int sock;
	struct addrinfo meta;
} tubcli_t;

const char *tub_errstr(int err);

int tub_serv_make (tubserv_t *s, uint16_t port, tubnet_t mode, tubip_t ip);

int tub_serv_close (tubserv_t *s);

int tub_cli_make (tubcli_t *c, tubnet_t mode, tubip_t ip);

int tub_cli_connect (tubcli_t *c, uint16_t port, const char *addr);

int tub_cli_close (tubcli_t *c);

#endif /* tubuil_h */
