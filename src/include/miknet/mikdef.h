#ifndef MIKNET_MIKDEF_H_
#define MIKNET_MIKDEF_H_

typedef enum {
	MIK_FALSE = 0,
	MIK_TRUE = 1
} mikbool_t;

typedef enum {
	MIKERR_NONE = 0,
	MIKERR_BAD_PTR = -1,
	MIKERR_LOOKUP = -2,
	MIKERR_CONNECT = -3,
	MIKERR_BAD_ADDR = -4,
	MIKERR_SOCKET = -5,
	MIKERR_NO_SUCH_FRAG = -6,
	MIKERR_BAD_MEM = -7
} mikerr_t;

#endif /* MIKNET_MIKDEF_H_ */
