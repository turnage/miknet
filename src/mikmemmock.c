#include <stdlib.h>

#include "miknet/mikmemmock.h"

static void *calloc_return = NULL;
static void *realloc_return = NULL;

static mikbool_t calloc_uses_system = MIK_FALSE;
static mikbool_t realloc_uses_system = MIK_FALSE;
static mikbool_t free_uses_system = MIK_FALSE;

void mikmemmock_reset()
{
	calloc_return = NULL;
	realloc_return = NULL;

	calloc_uses_system = MIK_FALSE;
	realloc_uses_system = MIK_FALSE;
	free_uses_system = MIK_FALSE;
}

void mik_mock_calloc_set_return(void *ptr)
{
	calloc_return = ptr;
}

void mik_mock_calloc_use_system(mikbool_t mode)
{
	calloc_uses_system = mode;
}

void *mik_mock_calloc(size_t num_elems, size_t elem_size)
{
	if (calloc_uses_system)
		return calloc(num_elems, elem_size);
	return calloc_return;
}

void mik_mock_realloc_set_return(void *ptr)
{
	realloc_return = ptr;
}

void mik_mock_realloc_use_system(mikbool_t mode)
{
	realloc_uses_system = mode;
}

void *mik_mock_realloc(void *ptr, size_t new_size)
{
	if (realloc_uses_system)
		return realloc(ptr, new_size);
	return realloc_return;
}

void mik_mock_free_use_system(mikbool_t mode)
{
	free_uses_system = mode;
}

void mik_mock_free(void *ptr)
{
	if (free_uses_system)
		free(ptr);
}
