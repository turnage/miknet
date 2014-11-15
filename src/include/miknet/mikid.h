#ifndef MIKNET_MIKID_H_
#define MIKNET_MIKID_H_

#include <stdint.h>

/**
 *  Returns an identifier. This identifier is guarunteed not to have been given
 *  to any preceding caller. No future callers will receive it either.
 */
uint16_t mikid();

#endif  /* MIKNET_MIKID_H_ */
