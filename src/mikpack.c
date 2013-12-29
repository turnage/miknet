#include <miknet/miknet.h>

mikpack_t mikpack (miktype_t type, ref *data, uint16_t len, uint32_t channel)
{
	mikpack_t pack = {0};

	pack.type = type;
	pack.channel = channel;

	if (!data) {
		pack.data = NULL;
		pack.len = 0;
	} else {
		pack.len = len;
		pack.data = try_alloc(pack.data, pack.len);
		memcpy(pack.data, data, pack.len);
	}

	return pack;
}