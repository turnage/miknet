#include "miknet/mikmemapi.h"
#include "miknet/mikmemshield.h"

mikmemshield_t mikmemshield_initialize()
{
	return mikmemshield_initialize_from(mik_calloc, mik_realloc, mik_free);
}

mikmemshield_t mikmemshield_initialize_from(
					void *(*calloc_cb) (size_t, size_t),
					void *(*realloc_cb) (void *, size_t),
					void (*free_cb) (void *))
{
	mikmemshield_t shield = {NULL};

	shield.calloc = calloc_cb;
	shield.realloc = realloc_cb;
	shield.free = free_cb;

	return shield;
}
