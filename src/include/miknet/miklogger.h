#ifndef MIKLOGGER_H_
#define MIKLOGGER_H_

typedef enum {
	MIK_LOG_NONE = -1,
	MIK_LOG_FATAL = 0,
	MIK_LOG_ERROR = 1,
	MIK_LOG_VERBOSE = 2
} mikloglevel_t;

/**
 *  Space: User space.
 *
 *  Toggles the log level of miklogger. Whatever the level, all messages of
 *  that level and below will be logged.
 */
void mik_log_set_level(mikloglevel_t new_level);

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
