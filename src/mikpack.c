#include <miknet/miknet.h>

mikpack_t mikpack (miktype_t type, void *data, uint16_t len)
{
	mikpack_t pack;

	pack.meta = type;
	pack.len = len;
	pack.data = data;

	if (!pack.data)
		pack.len = 0;

	return pack;
}