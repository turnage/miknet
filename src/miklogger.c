#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "miknet/miklogger.h"

static const char *MIK_LOG_PREFIXES[] = {"FATAL: ", "ERROR: ", "INFO: "};
static const int MIK_LOG_PREFIX_LENGTHS[] = {7, 7, 6};
static mikloglevel_t loglevel = MIK_LOG_VERBOSE;

void mik_log_set_level(mikloglevel_t new_level)
{
	loglevel = new_level;
}

void mik_log(mikloglevel_t level, const char *text, ...)
{
	va_list list;

	va_start(list, text);
	mik_log_core(level, NULL, text, list);
}

void mik_log_core(mikloglevel_t level, char *dest, const char *text, ...)
{
	va_list list;

	if (loglevel < level)
		return;

	if (text == NULL) {
		mik_log_core(MIK_LOG_ERROR, dest, "Attempted to log NULL.\n");
		return;
	}

	va_start(list, text);
	if (dest != NULL) {
		sprintf(dest, MIK_LOG_PREFIXES[level]);
		vsprintf(dest + MIK_LOG_PREFIX_LENGTHS[level], text, list);
	} else {
		fprintf(stderr, MIK_LOG_PREFIXES[level]);
		vfprintf(stderr, text, list);
	}
}
