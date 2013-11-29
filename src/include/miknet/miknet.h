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
#define MIK_IPST_MAX 48
#define MIK_WAIT_MAX 64
#define MIK_PEER_MAX 100

#define MIK_DEBUG 1

enum {
	ERR_MISSING_PTR  = -1,
	ERR_INVALID_MODE = -2,
	ERR_INVALID_IP   = -3,
	ERR_SOCKET       = -4,
	ERR_ADDRESS      = -5,
	ERR_SOCK_OPT     = -6,
	ERR_BIND         = -7,
	ERR_CONNECT      = -8,
	ERR_PEER_MAX     = -9,
	ERR_POLL         = -10,
        ERR_MEMORY       = -11
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
	struct sockaddr_storage addr;
	socklen_t addrlen;
	char ipst[MIK_IPST_MAX];
	uint32_t sent;
	uint32_t recvd;
	struct mikpeer_t *prev;
	struct mikpeer_t *next;
} mikpeer_t;

typedef struct mikpack_t {
	uint16_t len;
	char data[MIK_PACK_MAX];
} mikpack_t;

typedef struct mikserv_t {
	int sock;
	struct sockaddr_storage addr;
	socklen_t addrlen;
	miknet_t mode;
	mikip_t ip;
	struct pollfd *fds;
	nfds_t nfds;
	mikpeer_t *peers;
	uint16_t peerc;
	uint16_t peermax;
	uint32_t upcap;
	uint32_t downcap;
} mikserv_t;

typedef struct mikcli_t {
	int sock;
	struct addrinfo meta;
	char ipst[MIK_IPST_MAX];
	miknet_t mode;
	miknet_t ip;
} mikcli_t;

void mik_print_addr(struct sockaddr *addr, socklen_t l);

int mik_tcp_peer(mikserv_t *s);

int mik_tcp_poll(mikserv_t *s, int t);

const char *mik_errstr(int err);

int mik_serv_make (mikserv_t *s, uint16_t port, miknet_t mode, mikip_t ip);

int mik_serv_config (mikserv_t *s, uint16_t pm, uint32_t u, uint32_t d);

int mik_serv_poll (mikserv_t *s, int t);

int mik_serv_close (mikserv_t *s);

int mik_cli_make (mikcli_t *c, miknet_t mode, mikip_t ip);

int mik_cli_connect (mikcli_t *c, uint16_t port, const char *addr);

int mik_cli_close (mikcli_t *c);

#endif /* miknet_h */
