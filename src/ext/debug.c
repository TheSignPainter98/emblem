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

#ifndef MAX_DEBUG_STR_LEN
#	define MAX_DEBUG_STR_LEN 0
#endif
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
					const char* s = luaL_tolstring(L, i, &s_len);
#if MAX_DEBUG_STR_LEN <= 0
					fprintf(stderr, "'%s'\n", s);
#else
					char* t = strdup(s);
					if (s_len >= MAX_DEBUG_STR_LEN)
						strcpy(t + MAX_DEBUG_STR_LEN - strlen(ELIPSES_STRING), ELIPSES_STRING); // NOLINT
					fprintf(stderr, "'%s'\n", t);
					free(t);
#endif
					lua_pop(L, 1);
					break;
				}
			}
		}
}
