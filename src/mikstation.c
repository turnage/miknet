#include <errno.h>
#include <stddef.h>
#include <stdlib.h>

#include "miknet/mikstation.h"

#include "miknet/mikdef.h"

static ssize_t mikstation_parse_error(ssize_t error)
{
	if (error == -1)
		if (errno == EWOULDBLOCK)
			return MIKERR_NO_MSG;
		else
			return MIKERR_NET_FAIL;
	else
		return error;
}

static ssize_t mikstation_blank_read(	const int sockfd,
					const posix_t *posix,
					const int flags)
{
	char bin;

	if (sockfd < 0)
		return MIKERR_BAD_VALUE;

	if (posix == NULL)
		return MIKERR_BAD_PTR;

	return mikstation_parse_error(posix->recvfrom(	posix,
							sockfd,
							&bin,
							1,
							flags,
							NULL,
							NULL));
}

ssize_t mikstation_discard(const int sockfd, const posix_t *posix)
{
	return mikstation_blank_read(sockfd, posix, MSG_TRUNC);
}

ssize_t mikstation_poll(const int sockfd, const posix_t *posix)
{
	return mikstation_blank_read(sockfd, posix, MSG_PEEK | MSG_TRUNC);
}

int mikstation_receive(	const int sockfd,
			const posix_t *posix,
			mikgram_t *gram,
			mikaddr_t *addr)
{
	ssize_t error;

	if (sockfd < 0)
		return MIKERR_BAD_VALUE;

	if (posix == NULL || gram == NULL || addr == NULL)
		return MIKERR_BAD_PTR;

	error = mikstation_poll(sockfd, posix);
	if (error < 0)
		return error;

	gram->len = error;
	gram->data = malloc(gram->len);
	if (gram->data == NULL)
		return MIKERR_BAD_MEM;

	error = mikstation_parse_error(posix->recvfrom(	posix,
							sockfd,
							gram->data,
							gram->len,
							0,
							&addr->addr,
							&addr->addrlen));

	if (error < 0 || error < gram->len) {
		free(gram->data);
		return error;
	}

	return 0;
}

int mikstation_send(	const int sockfd,
			const posix_t *posix,
			const mikgram_t *gram,
			const mikaddr_t *addr)
{
	ssize_t error;

	if (posix == NULL || gram == NULL || addr == NULL)
		return MIKERR_BAD_PTR;

	if (gram->data == NULL)
		return MIKERR_BAD_PTR;

	if (sockfd < 0)
		return MIKERR_BAD_VALUE;

	if (gram->len < MIKNET_GRAM_MIN_SIZE)
		return MIKERR_BAD_VALUE;

	error = mikstation_parse_error(posix->sendto(	posix,
							sockfd,
							gram->data,
							gram->len,
							0,
							&addr->addr,
							addr->addrlen));

	if (error < 0)
		return error;

	if (error < gram->len)
		return MIKERR_NET_FAIL;

	return 0;
}
