#include <miknet/miknet.h>

mikpack_t mikpack (miktype_t type, void *data, uint16_t len)
{
	mikpack_t pack;

	pack.type = type;

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