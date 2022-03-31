/**
 * @file location.c
 * @brief Implements the Location data-structure and useful functions for keeping track of locations in the document
 * source
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "location.h"

#include "ext/debug.h"
#include "ext/lua.h"
#include <lauxlib.h>
#include <lua.h>
#include <string.h>

#define EM_UNPACK_LOC_FUNC_NAME "unpack_loc"
#define EM_GET_LOC_ID_FUNC_NAME "__get_loc_id"

static int ext_copy_location(ExtensionState* s);

Location* dup_loc(Location* todup)
{
	Location* ret = malloc(sizeof(Location));
	memcpy(ret, todup, sizeof(Location));
	ret->id			   = get_unique_id();
	return ret;
}

void register_ext_location(ExtensionState* s)
{
	register_api_function(s, EM_UNPACK_LOC_FUNC_NAME, ext_unpack_location);
	register_api_function(s, EM_GET_LOC_ID_FUNC_NAME, ext_get_location_id);
}

static int ext_unpack_location(ExtensionState* s)
{
	luaL_argcheck(s, lua_isuserdata(s, 1), 1, "Function " EM_UNPACK_LOC_FUNC_NAME " expected location to unpack");
	Location* loc;
	if (to_userdata_pointer((void**)&loc, s, 1, LOCATION))
		return luaL_error(s, "Failed to unpack lua pointer");

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

static int ext_get_location_id(ExtensionState* s)
{
	luaL_argcheck(s, lua_isuserdata(s, 1), 1, "Expected location");
	Location* loc;
	if (to_userdata_pointer((void**)&loc, s, 1, LOCATION))
		return luaL_error(s, "Failed to unpack lua pointer");
	lua_pushinteger(s, LOC_ID(loc));
	return 1;
}

void push_location_lua_pointer(ExtensionState* s, Location* loc)
{
	get_api_elem(s, "locs");
	lua_pushinteger(s, LOC_ID(loc));
	lua_gettable(s, -2);
	if (lua_isnil(s, -1))
	{
		loc->refs++;
		new_lua_pointer(s, LOCATION, loc, true);

		// Save into loc ptr table
		lua_rotate(s, -2, 1);
		lua_settable(s, -3);

		// Return the loc obj
		lua_pushinteger(s, LOC_ID(loc));
		lua_gettable(s, -2);
	}
	lua_remove(s, -2);
}
