#ifndef miknet_h
#define miknet_h

#include <stdio.h>
#include <errno.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include <poll.h>
#include <netdb.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <netinet/in.h>

#include <sys/socket.h>
#include <sys/types.h>
#include <sys/time.h>

#define MIK_PEER_MAX 100
#define MIK_CHAN_MAX 100
#define MIK_PACK_MAX 1200
#define MIK_PORT_MAX 6
#define MIK_LIST_MAX 100

#define MIK_DEBUG 1

enum {
	ERR_MISSING_PTR  = -1,
	ERR_INVALID_MODE = -2,
	ERR_SOCKET       = -4,
	ERR_ADDRESS      = -5,
	ERR_SOCK_OPT     = -6,
	ERR_BIND         = -7,
	ERR_CONNECT      = -8,
	ERR_PEER_MAX     = -9,
	ERR_POLL         = -10,
        ERR_MEMORY       = -11,
	ERR_WOULD_FAULT  = -12,
	ERR_LISTEN       = -13
};

typedef enum {
	MIK_DISC = 0,
	MIK_CONN = 2
} mikstate_t;

typedef enum {
	MIK_FAST = 1,
	MIK_UDP  = 1,
	MIK_SAFE = 2,
	MIK_TCP  = 2
} miknet_t;

typedef enum {
	MIK_IPV4 = 1,
	MIK_IPV6 = 2
} mikip_t;

typedef enum {
	MIK_ERR  = -1,
	MIK_INIT = 0,
	MIK_QUIT = 1,
	MIK_DATA = 2
} miktype_t;

typedef struct mikpeer_t {
	int tcp;
	struct sockaddr_storage addr;
	socklen_t addrlen;
	mikstate_t state;
	uint32_t sent;
	uint32_t recvd;
} mikpeer_t;

typedef struct mikpack_t {
	miktype_t meta;
	uint16_t len;
	void *data;
} mikpack_t;

typedef struct miknode_t {
	int tcp;
	int udp;
	mikip_t ip;
	struct pollfd *fds;
	mikpeer_t *peers;
	uint16_t peerc;
	uint16_t peermax;
	mikpack_t *packs;
	uint32_t upcap;
	uint32_t downcap;
} miknode_t;

int mik_debug (int err);

int mik_print_addr(struct sockaddr *addr, socklen_t s);

const char *mik_errstr(int err);

int mik_bind (int *t, int *u, struct addrinfo h, uint16_t p);

int miknode (miknode_t *n, mikip_t ip, uint16_t port);

#endif /* miknet_h */
