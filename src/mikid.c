#include "miknet/mikid.h"

uint16_t mikid()
{
	static uint16_t id = 0;
	return id++;
}

