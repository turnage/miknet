#ifndef MIKLOGGER_H_
#define MIKLOGGER_H_

/* mikogger is _NOT_ a userspace module. These functions can change at any
   time with no notice. Do _NOT_ use them. */

typedef enum {
	MIK_LOG_INFO = 0,
	MIK_LOG_TRIP = 1,
	MIK_LOG_FATAL = 2
} mikloglevel_t;

/**
 *  Logs some debug information to stderr.
 *
 *  This takes format strings, just like printf.
 */
void mik_log(mikloglevel_t level, const char *text, ...);

void mik_log_core(mikloglevel_t level, char *dest, const char *text, ...);

#endif  /* MIKLOGGER_H_ */
