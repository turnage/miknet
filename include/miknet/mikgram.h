#ifndef MIKNET_MIKGRAM_H_
#define MIKNET_MIKGRAM_H_

/**
 *  A mikgram represents the most basic unit of communication between miknodes.
 *  It abbreviates "miknode datagram".
 */
typedef struct mikgram_t {
	void *data;
	size_t len;
} mikgram_t;

/**
 *  Creates a complete mikgram for a chunk of data, with its own copy. A mikgram
 *  contains the original data and a serialized header containing its size and
 *  some flags.
 *
 *  mikgrams must be disposed of with mikgram_close.
 */
int mikgram(mikgram_t *gram, const void *data, size_t len);

/**
 *  If the data is a complete mikgram, returns the number of octets needed to
 *  extract the payload from it.
 *
 *  Returns zero if the data is not a mikgram, or if the mikgram is incomplete,
 *  or a negative value on error.
 */
ssize_t mikgram_check(const mikgram_t *gram);

/**
 *  Extracts the data from a packed/serialized mikgram into a buffer.
 */
int mikgram_extract(const mikgram_t *gram, void *buf, size_t len);

/**
 *  Frees the resources used by a mikgram.
 */
void mikgram_close(mikgram_t *gram);

#endif /* MIKNET_MIKGRAM_H_ */
