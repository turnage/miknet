#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "miknet/miklogger.h"

static const char *MIK_LOG_PREFIXES[] = {"INFO: ", "TRIPPING: ", "FATAL: "};
static const int MIK_LOG_PREFIX_LENGTHS[] = {6, 10, 7};
static miklogstate_t logstate = MIK_LOG_ON;

void mik_log_toggle(miklogstate_t new_logstate)
{
	logstate = new_logstate;
}

void mik_log(mikloglevel_t level, const char *text, ...)
{
	va_list list;

	va_start(list, text);
	mik_log_core(level, NULL, text, list);
}

void mik_log_core(mikloglevel_t level, char *dest, const char *text, ...)
{
	if (logstate == MIK_LOG_OFF)
		return;

	if (text == NULL) {
		mik_log_core(MIK_LOG_TRIP, dest, "Attempted to log NULL.\n");
		return;
	}

	const char *prefix = MIK_LOG_PREFIXES[level];
	int prefix_offset = MIK_LOG_PREFIX_LENGTHS[level];
	va_list list;

	va_start(list, text);
	if (dest != NULL) {
		sprintf(dest, prefix);
		vsprintf(dest + prefix_offset, text, list);
	} else {
		fprintf(stderr, prefix);
		vfprintf(stderr, text, list);
	}
}
