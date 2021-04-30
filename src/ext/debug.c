#include "debug.h"

#include "logs/logs.h"
#include <lauxlib.h>
#include <stdio.h>

void dumpstack(lua_State* L)
{
	int top = lua_gettop(L);
	for (int i = 1; i <= top; i++)
	{
		log_debug("|> %d\t%s:\t", i, luaL_typename(L, i));
		switch (lua_type(L, i))
		{
			case LUA_TNUMBER:
				log_debug("\033[1A\t\t\t\t\t\t%g", lua_tonumber(L, i));
				break;
			case LUA_TSTRING:
				log_debug("\033[1A\t\t\t\t\t\t%s", lua_tostring(L, i));
				break;
			case LUA_TBOOLEAN:
				log_debug("\033[1A\t\t\t\t\t\t%s", (lua_toboolean(L, i) ? "true" : "false"));
				break;
			case LUA_TNIL:
				log_debug("\033[1A\t\t\t\t\t\t%s", "");
				break;
			default:
				log_debug("\033[1A\t\t\t\t\t%p", lua_topointer(L, i));
				break;
		}
	}
}
