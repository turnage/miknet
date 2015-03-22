#ifndef MIKNET_MIKPEER_H_
#define MIKNET_MIKPEER_H_

#include "miknet/mikaddr.h"

typedef struct mikpeer_t {
	uint8_t exists;
	mikaddr_t address;
	void *data;
} mikpeer_t;

#endif /* MIKNET_MIKPEER_H_ */
