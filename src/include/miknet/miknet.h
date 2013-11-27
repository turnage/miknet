#ifndef miknet_h
#define miknet_h

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

#define MIK_PEER_MAX 100
#define MIK_CHAN_MAX 100
#define MIK_PACK_MAX 1200
#define MIK_PORT_MAX 6
#define MIK_WAIT_MAX 64

#define MIK_DEBUG 1

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
	MIK_FAST = 1,
	MIK_UDP  = 1,
	MIK_SAFE = 2,
	MIK_TCP  = 3
} miknet_t;

typedef enum {
	MIK_IPV4 = 1,
	MIK_IPV6 = 2
} mikip_t;

typedef struct mikpeer_t {
	int sock;
	struct mikpeer_t *prev;
	struct mikpeer_t *next;
} mikpeer_t;

typedef struct mikpack_t {
	uint16_t len;
	char data[MIK_PACK_MAX];
} mikpack_t;

typedef struct mikserv_t {
	int sock;
	struct sockaddr address;
	miknet_t socktype;
	mikip_t iptype;
} mikserv_t;

typedef struct mikcli_t {
	int sock;
	struct addrinfo meta;
} mikcli_t;

const char *mik_errstr(int err);

int mik_serv_make (mikserv_t *s, uint16_t port, miknet_t mode, mikip_t ip);

int mik_serv_close (mikserv_t *s);

int mik_cli_make (mikcli_t *c, miknet_t mode, mikip_t ip);

int mik_cli_connect (mikcli_t *c, uint16_t port, const char *addr);

int mik_cli_close (mikcli_t *c);

#endif /* miknet_h */
