#include <stdlib.h>

#include "miknet/mikmsg.h"
#include "miknet/mikdef.h"

mikmsg_t *mikmsg(const mikgram_t *gram, const mikaddr_t *addr)
{
	mikmsg_t *msg;
	ssize_t payload_len;

	if (gram == NULL || addr == NULL)
		return NULL;

	if (gram->data == NULL)
		return NULL;

	payload_len = mikgram_check(gram);
	if (payload_len <= 0)
		return NULL;

	msg = malloc(sizeof(mikmsg_t) + payload_len);
	if (msg == NULL)
		return NULL;

	msg->data = (void *)msg + sizeof(mikmsg_t);
	if (mikgram_extract(gram, msg->data, payload_len) != MIK_SUCCESS) {
		mikmsg_close(msg);
		return NULL;
	}

	msg->len = payload_len;
	msg->addr = *addr;
	msg->next = NULL;

	return msg;
}

void mikmsg_close(mikmsg_t *msg)
{
	free(msg);
}
