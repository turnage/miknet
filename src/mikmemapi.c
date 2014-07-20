#include "miknet/mikmemapi.h"

void *mik_calloc(size_t num_elems, size_t elem_size)
{
	return calloc(num_elems, elem_size);
}

void *mik_realloc(void *ptr, size_t new_size)
{
	return realloc(ptr, new_size);
}

void mik_free(void *ptr)
{
	free(ptr);
}

