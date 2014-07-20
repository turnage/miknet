#ifndef MIKLOGGER_H_
#define MIKLOGGER_H_

typedef enum {
	MIK_LOG_INFO = 0,
	MIK_LOG_TRIP = 1,
	MIK_LOG_FATAL = 2
} mikloglevel_t;

typedef enum {
	MIK_LOG_ON = 0,
	MIK_LOG_OFF = 1
} miklogstate_t;

/**
 *  Space: User space.
 *
 *  Toggles the state of miklogger, on and off. See miklogstate_t.
 */
void mik_log_toggle(miklogstate_t new_logstate);

/**
 *  Space: Internal only.
 *
 *  Logs some debug information to stderr.
 *  This takes format strings, just like printf.
 */
void mik_log(mikloglevel_t level, const char *text, ...);

/**
 *  Space: Internal only.
 *
 *  Log some debug information to the provided destination in memory, or stderr
 *  if that is NULL. This takes format strings, just like printf.
 */
void mik_log_core(mikloglevel_t level, char *dest, const char *text, ...);

#endif  /* MIKLOGGER_H_ */
