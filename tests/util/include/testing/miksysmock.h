#ifndef MIKNET_MIKSYSMOCK_H_
#define MIKNET_MIKSYSMOCK_H_

#include "miknet/miksys.h"

/**
 *  Initializes miksysmock. After calling this, calls to the miksys functions
 *  will instead call the mock functions.
 */
void miksysmock_init();

/**
 *  Sets the value that mock functions will return when called, if a mock
 *  function returns a value.
 */
void miksysmock_set_return(int value);

/**
 *  Defines the value a mutable argument will be set to if a mock function has a 
 *  mutable argument.
 */
void miksysmock_set_arg(uint64_t value);

#endif /* MIKNET_MIKSYSMOCK_H_ */
