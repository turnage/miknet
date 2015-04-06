#ifndef MIKNET_MIKDEF_H_
#define MIKNET_MIKDEF_H_

#include <limits.h>

typedef enum {
	MIK_FALSE = 0,
	MIK_TRUE = 1
} mikbool_t;

typedef enum {
	MIKERR_NONE = 0,
	MIKERR_VALUE_BOUND = INT_MIN,

	/* Errors beneath miknet. */
	MIKERR_BAD_MEM,
	MIKERR_BAD_PTR,
	MIKERR_NET_FAIL,
	MIKERR_SYS_FAIL,

	/* Argument errors. */
	MIKERR_BAD_VALUE,

	/* Protocol errors. */
	MIKERR_NO_MSG,
	MIKERR_NONCONFORM
} mikerr_t;

#endif /* MIKNET_MIKDEF_H_ */
