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
#include <netinet/in.h>

#include <sys/socket.h>
#include <sys/time.h>

#define MIK_PACK_MAX 1200
#define MIK_PORT_MAX 6
#define MIK_MEMEXP   100

struct miknode_t;
typedef const void ref;

/* error codes */
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

/* miknode flags */
enum {
	/* do not use Miknet protocol */
	MIK_FLAG_NOPROTO = 1,
};

typedef enum {
	MIK_DISC = 0,
	MIK_CONN = 2
} mikstate_t;

typedef enum {
	MIK_IPV4 = 1,
	MIK_IPV6 = 2
} mikip_t;

typedef enum {
	MIK_ERR  = -1,
	MIK_JOIN = 0,
	MIK_QUIT = 1,
	MIK_DATA = 2
} miktype_t;

typedef struct mikpack_t {
	miktype_t type;
	uint32_t channel;
	uint16_t peer;
	uint16_t len;
	void *data;
} mikpack_t;

typedef struct mikvec_t {
	size_t size;
	size_t memsize;
	int index;
	int rs_mall; /* rounds since malloc */
	uint64_t total_size; /* cumulative; counts and resets with rs_mall */
	mikpack_t *data;
} mikvec_t;

typedef struct mikpeer_t {
	int index;
	struct miknode_t *node;
	int tcp;
	uint32_t flags;
	void *data;
	mikstate_t state;
	uint32_t sent;
	uint32_t recvd;
} mikpeer_t;

typedef struct miknode_t {
	int tcp;
	uint32_t flags;
	mikip_t ip;
	struct pollfd *fds;
	mikpeer_t *peers;
	uint16_t peerc;
	uint16_t peermax;
	mikvec_t packs;
	mikvec_t commands;
} miknode_t;

int mik_debug (int err);

void *try_alloc(void *ptr, size_t bytes);

mikpack_t *mikevent (miknode_t *node);

const char *mik_errstr(int err);

mikpack_t mikpack (miktype_t type, ref *data, uint16_t len, uint32_t channel);

mikvec_t mikvec(mikpack_t data);

mikvec_t mikvec_add (mikvec_t vector, mikpack_t data);

mikpack_t *mikvec_next (mikvec_t *vector);

mikvec_t mikvec_clear (mikvec_t vector);

mikvec_t mikvec_close (mikvec_t vector);

int miknode_set_flags(miknode_t *n, unsigned int flags);
int miknode_unset_flags(miknode_t *n, unsigned int flags);
int miknode_check_flags(miknode_t *n, unsigned int flags);

int miknode (miknode_t *n, mikip_t ip, uint16_t port, uint16_t peers);

int miknode_connect(miknode_t *n, const char *a, uint16_t p);

int miknode_send (mikpeer_t *p, ref *d, size_t len, uint32_t channel);

int miknode_poll (miknode_t *n, int t);

void miknode_close (miknode_t *n);

int mikpeer (miknode_t *n);

int mikpeer_close (mikpeer_t *p);

int mikpeer_set_flags(mikpeer_t *n, unsigned int flags);
int mikpeer_unset_flags(mikpeer_t *n, unsigned int flags);
int mikpeer_check_flags(mikpeer_t *n, unsigned int flags);

#endif /* miknet_h */
