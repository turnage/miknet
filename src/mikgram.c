#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "miknet/mikdef.h"
#include "miknet/mikgram.h"

int mikgram(mikgram_t *gram, const void *data, size_t len)
{
	if (gram == NULL || data == NULL)
		return MIKERR_BAD_PTR;

	if (len == 0 || len > MIKNET_GRAM_MAX_SIZE)
		return MIKERR_BAD_VALUE;

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

ssize_t mikgram_check(const mikgram_t *gram)
{
	ssize_t payload_len;

	if (gram == NULL)
		return MIKERR_BAD_PTR;

	if (gram->data == NULL)
		return MIKERR_BAD_PTR;

	payload_len =	((uint8_t *)gram->data)[0]
			^ (((uint8_t *)gram->data)[1] << 8);

	if (gram->len < payload_len + MIKNET_METADATA_SIZE)
		return MIKERR_BAD_VALUE;

	return payload_len;
}

int mikgram_extract(const mikgram_t *gram, void *buf, size_t len)
{
	if (gram == NULL)
		return MIKERR_BAD_PTR;

	if (gram->data == NULL || buf == NULL)
		return MIKERR_BAD_PTR;

	if (gram->len == 0 || len < gram->len - MIKNET_METADATA_SIZE)
		return MIKERR_BAD_VALUE;

	memcpy(	buf,
		gram->data + MIKNET_METADATA_SIZE,
		gram->len - MIKNET_METADATA_SIZE);

	return MIKERR_NONE;
}

void mikgram_close(mikgram_t *gram)
{
	free(gram->data);
}
