#ifndef MIKNET_MIKMEMMOCK_H_
#define MIKNET_MIKMEMMOCK_H_

#include "miknet/mikdef.h"

/**
 *  Space: Internal only.
 *
 *  Resets all mikmemmock member variables.
 */
void mikmemmock_reset();

/**
 *  Space: Internal only.
 *
 *  Sets the value mik_mock_calloc() will return.
 */
void mik_mock_calloc_set_return(void *ptr);

/**
 *  Space: Internal only. 
 *
 *  Toggles whether the dummy method will call the system calloc.
 */
void mik_mock_calloc_use_system(mikbool_t mode);

/**
 *  Space: Internal only.
 *
 *  Dummy calloc function.
 */
void *mik_mock_calloc(size_t num_elems, size_t elem_size);

/**
 *  Space: Internal only.
 *
 *  Sets the value mik_mock_realloc() will return.
 */
void mik_mock_realloc_set_return(void *ptr);

/**
 *  Space: Internal only. 
 *
 *  Toggles whether the dummy method will call the system realloc.
 */
void mik_mock_realloc_use_system(mikbool_t mode);

/**
 *  Space: Internal only.
 *
 *  Dummy realloc function.
 */
void *mik_mock_realloc(void *ptr, size_t new_size);

/**
 *  Space: Internal only. 
 *
 *  Toggles whether the dummy method will call the system free.
 */
void mik_mock_free_use_system(mikbool_t mode);

/**
 *  Space: Internal only.
 *
 *  Dummy free function.
 */
void mik_mock_free(void *ptr);

#endif  /* MIKNET_MIKMEMMOCK_H_ */
