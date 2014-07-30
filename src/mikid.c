#include "miknet/mikid.h"

static uint64_t id = 0;

uint64_t mikid()
{
	return id++;
}

