#include <miknet/miknet.h>

mikpack_t mikpack (miktype_t type, void *data, uint16_t len)
{
	mikpack_t pack;

	pack.meta = type;
	pack.len = len;
	pack.data = data;

	return pack;
}