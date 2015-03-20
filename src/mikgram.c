#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "miknet/mikdef.h"
#include "miknet/mikgram.h"

#define MIKNET_GRAM_MAX_SIZE	512
#define MIKNET_METADATA_SIZE	4

int mikgram(mikgram_t *gram, const void *data, size_t len)
{
	if (gram == NULL || data == NULL)
		return MIKERR_BAD_PTR;

	if (len == 0)
		return MIKERR_BAD_LENGTH;

	if (len > MIKNET_GRAM_MAX_SIZE)
		return MIKERR_GRAM_SIZE;

	gram->len = MIKNET_METADATA_SIZE + len;
	gram->data = malloc(gram->len);
	if (gram->data == NULL)
		return MIKERR_BAD_MEM;

	((uint8_t *)gram->data)[0] = len & 0xff;
	((uint8_t *)gram->data)[1] = (len >> 8) & 0xff;

	/* Reserved space. */
	((uint8_t *)gram->data)[2] = 0;
	((uint8_t *)gram->data)[3] = 0;

	memcpy(gram->data + MIKNET_METADATA_SIZE, data, len);

	return MIKERR_NONE;
}

ssize_t mikgram_check(const void *data, size_t len)
{
	ssize_t payload_len;

	if (data == NULL)
		return MIKERR_BAD_PTR;

	payload_len = ((uint8_t *)data)[0] ^ (((uint8_t *)data)[1] << 8);

	if (len < payload_len + MIKNET_METADATA_SIZE)
		return MIKERR_BAD_LENGTH;

	return payload_len;
}

int mikgram_extract(const void *data, size_t datalen, void *buf, size_t len)
{
	if (data == NULL || buf == NULL)
		return MIKERR_BAD_PTR;

	if (datalen == 0 || len < datalen - MIKNET_METADATA_SIZE)
		return MIKERR_BAD_LENGTH;

	memcpy(	buf,
		data + MIKNET_METADATA_SIZE,
		datalen - MIKNET_METADATA_SIZE);

	return MIKERR_NONE;
}

void mikgram_close(mikgram_t *gram)
{
	free(gram->data);
}
