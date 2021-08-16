#include "debug.h"

#include "logs/logs.h"
#include <lauxlib.h>
#include <stdio.h>

void _dumpstack(lua_State* L)
{
	int top = lua_gettop(L);
	if (!top)
		fputs("|> <>\n", stderr);
	else
		for (int i = 1; i <= top; i++)
		{
			fprintf(stderr, "|> %d\t%s:\t", i, luaL_typename(L, i));
			switch (lua_type(L, i))
			{
				case LUA_TNUMBER:
					fprintf(stderr, "%g\n", lua_tonumber(L, i));
					break;
				case LUA_TSTRING:
					fprintf(stderr, "'%s'\n", lua_tostring(L, i));
					break;
				case LUA_TBOOLEAN:
					fprintf(stderr, "%s\n", (lua_toboolean(L, i) ? "true" : "false"));
					break;
				case LUA_TNIL:
					fprintf(stderr, "%s\n", "");
					break;
				default:
					fprintf(stderr, "'%s'\n", luaL_tolstring(L, i, NULL));
					lua_pop(L, 1);
					break;
			}
		}
}
