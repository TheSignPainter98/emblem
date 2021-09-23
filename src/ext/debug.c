/**
 * @file debug.c
 * @brief Implements basic functionality for debugging involving the Lua stack
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "debug.h"

#include "logs/logs.h"
#include <lauxlib.h>
#include <stdio.h>
#include <string.h>

#define MAX_DEBUG_STR_LEN 60
#define ELIPSES_STRING "..."

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
				{
					size_t s_len;
					char* s = strdup(luaL_tolstring(L, i, &s_len));
					if (s_len >= MAX_DEBUG_STR_LEN)
						strcpy(s + MAX_DEBUG_STR_LEN - strlen(ELIPSES_STRING), ELIPSES_STRING); // NOLINT
					fprintf(stderr, "'%s'\n", s);
					lua_pop(L, 1);
					free(s);
					break;
				}
			}
		}
}
