/**
 * @file location.c
 * @brief Implements the Location data-structure and useful functions for keeping track of locations in the document source
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "location.h"

#include "ext/debug.h"
#include "ext/lua.h"
#include <lauxlib.h>
#include <lua.h>
#include <string.h>

#define EM_COPY_LOC_FUNC_NAME "copy_loc"

static int ext_copy_location(ExtensionState* s);

Location* dup_loc(Location* todup)
{
	Location* ret = malloc(sizeof(Location));
	memcpy(ret, todup, sizeof(Location));
	return ret;
}

void set_ext_location_globals(ExtensionState* s) { lua_register(s, "_copy_loc", ext_copy_location); }

static int ext_copy_location(ExtensionState* s)
{
	lua_getglobal(s, "get_var");
	lua_pushliteral(s, EM_LOC_NAME);
	if (lua_pcall(s, 1, 1, 0) != LUA_OK)
		luaL_error(s, "Failed to get " EM_LOC_NAME ": %s", lua_tostring(s, -1));

	if (!lua_isuserdata(s, -1))
		luaL_error(s, "Global " EM_LOC_NAME " is not a userdata variable, failed to copy location");

	LuaPointer* locp = lua_touserdata(s, -1);
	lua_pop(s, 1);

	if (!locp)
		luaL_error(s, "Attempted to unpack a location userdata outside when there is no location to be pointed to");
	if (locp->type != LOCATION)
		luaL_error(s,
			"Global " EM_LOC_NAME
			" has been changed! Expected a userdata object of type %d but got one of type %d instead",
			LOCATION, locp->type);
	Location* loc = locp->data;

	lua_createtable(s, 0, 5);
	lua_pushinteger(s, loc->first_line);
	lua_setfield(s, -2, "first_line");
	lua_pushinteger(s, loc->first_column);
	lua_setfield(s, -2, "first_column");
	lua_pushinteger(s, loc->last_line);
	lua_setfield(s, -2, "last_line");
	lua_pushinteger(s, loc->last_column);
	lua_setfield(s, -2, "last_column");
	lua_pushlstring(s, loc->src_file->str, loc->src_file->len);
	lua_setfield(s, -2, "src_file");
	return 1;
}
