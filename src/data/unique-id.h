#pragma once

#include <stddef.h>
#include <lualib.h>

typedef lua_Integer UniqueID;

UniqueID get_unique_id(void);
