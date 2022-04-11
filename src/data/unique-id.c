#include "unique-id.h"

#include "logs/logs.h"
#include "pp/assert.h"

static UniqueID id = (UniqueID)1;

UniqueID get_unique_id(void)
{
	if (id)
		return id++;
	log_err("Too many objects, ID uniqueness guarantee has been lost");
	exit(1);
}
