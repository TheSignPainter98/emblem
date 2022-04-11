#include "unique-id.h"

#include "logs/logs.h"
#include "pp/assert.h"
#include <lualib.h>

static void check_ext_int_widths(void) __attribute__((constructor));
static void check_ext_int_widths(void) { ASSERT(sizeof(lua_Integer) >= sizeof(void*)); }

static UniqueID id = (UniqueID)1;

UniqueID get_unique_id(void)
{
	POINTER_ARITH(if (id) return id++;);
	log_err("Too many objects, ID uniqueness guarantee has been lost");
	exit(1);
}
