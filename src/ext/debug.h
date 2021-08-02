#pragma once

#include <lua.h>

#ifdef DEBUG
#define dumpstack _dumpstack
#else
#define dumpstack(s)
#endif

void _dumpstack(lua_State* L);
