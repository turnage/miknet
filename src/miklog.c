#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "miknet/miklog.h"

static const char *MIKLOG_PREFIXES[] = {"FATAL: ", "ERROR: ", "INFO: "};
static const int MIKLOG_PREFIX_LENGTHS[] = {7, 7, 6};
static mikloglevel_t loglevel = MIKLOG_VERBOSE;

void miklog_set_level(mikloglevel_t new_level)
{
	loglevel = new_level;
}

void miklog(mikloglevel_t level, const char *text, ...)
{
	va_list list;

	va_start(list, text);
	miklog_core(level, NULL, text, list);
}

void miklog_core(mikloglevel_t level, char *dest, const char *text, ...)
{
	va_list list;

	if (loglevel < level)
		return;

	if (text == NULL) {
		miklog_core(MIKLOG_ERROR, dest, "Attempted to log NULL.\n");
		return;
	}

	va_start(list, text);
	if (dest != NULL) {
		sprintf(dest, MIKLOG_PREFIXES[level]);
		vsprintf(dest + MIKLOG_PREFIX_LENGTHS[level], text, list);
	} else {
		fprintf(stderr, MIKLOG_PREFIXES[level]);
		vfprintf(stderr, text, list);
	}
}
