#ifndef MIKNET_MIKMEMAPI_H_
#define MIKNET_MIKMEMAPI_H_

#include <stdlib.h>

/**
 *  Space: Internal only.
 *
 *  Alias for calloc.
 */
void *mik_calloc(size_t num_elems, size_t elem_size);

/**
 *  Space: Internal only.
 *
 *  Alias for realloc.
 */
void *mik_realloc(void *ptr, size_t new_size);

/**
 *  Space: Internal only.
 *
 *  Alias for free.
 */
void mik_free(void *ptr);

#endif  /* MIKNET_MEMAPI_H_ */
