#include <miknet/miknet.h>

mikpack_t mikpack (miktype_t type, void *data, uint16_t len, uint32_t channel)
{
	mikpack_t pack = {0};

	pack.type = type;
	pack.channel = channel;

	if (!data) {
		pack.data = NULL;
		pack.len = 0;
	} else {
		pack.len = len;
		pack.data = calloc(1, pack.len);
		memcpy(pack.data, data, pack.len);
	}

	return pack;
}