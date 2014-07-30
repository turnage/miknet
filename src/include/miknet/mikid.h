#ifndef MIKNET_MIKID_H_
#define MIKNET_MIKID_H_

#include <stdint.h>

/**
 *  Space: Internal only.
 *
 *  Returns an identifier. This identifier is guarunteed not to have been given
 *  to any preceding caller. No future callers will receive it either.
 */
uint64_t mikid();

#endif  /* MIKNET_MIKID_H_ */
