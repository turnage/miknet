#ifndef MIKNET_MIKMEMSHIELD_H_
#define MIKNET_MIKMEMSHIELD_H_

typedef struct {
	void *(*calloc) (size_t, size_t);
	void *(*realloc) (void *, size_t);
	void (*free) (void *);
} mikmemshield_t;

/**
 *  Space: Internal only. 
 *
 *  Initializes a memshield with the default memory api callbacks.
 */
mikmemshield_t mikmemshield_initialize();

/**
 *  Space: Internal only. 
 *
 *  Initializes a memshield with the provided memory api callbacks.
 */
mikmemshield_t mikmemshield_initialize_from(
					void *(*calloc_cb) (size_t, size_t),
					void *(*realloc_cb) (void *, size_t),
					void (*free_cb) (void *));

#endif /* MIKNET_MIKMEMSHEILD_H_ */
