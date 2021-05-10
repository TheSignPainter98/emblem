#include "location.h"

#include <string.h>

Location* dup_loc(Location* todup)
{
	Location* ret = malloc(sizeof(Location));
	memcpy(ret, todup, sizeof(Location));
	return ret;
}
