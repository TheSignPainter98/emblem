/**
 * @file ext-log.c
 * @brief Implements the C-side interface between extension-space and logging functions
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "ext-log.h"

#include "ext/ext-env.h"
#include "logs.h"
#include <lauxlib.h>
#include <lua.h>

#define DEFAULT_LOC "(extension-space)"

static int ext_log_err(ExtensionState* s);
static int ext_log_err_at(ExtensionState* s);
static int ext_log_warn(ExtensionState* s);
static int ext_log_warn_at(ExtensionState* s);
static int ext_log_info(ExtensionState* s);
static int ext_log_debug(ExtensionState* s);

void register_ext_logging(ExtensionState* s)
{
	register_api_function(s, "__log_err", ext_log_err);
	register_api_function(s, "__log_err_at", ext_log_err_at);
	register_api_function(s, "__log_warn", ext_log_warn);
	register_api_function(s, "__log_warn_at", ext_log_warn_at);
	register_api_function(s, "__log_info", ext_log_info);
	register_api_function(s, "__log_debug", ext_log_debug);
}

#define ID(x) x
#define EXIT_ON_ERR(x)                                                                                                 \
	if (x)                                                                                                             \
		luaL_error(s, "Warnings are fatal");
#define EXT_LOG_AT_FUNC(lvl, err_handler)                                                                              \
	static int ext_log_##lvl##_at(ExtensionState* s)                                                                   \
	{                                                                                                                  \
		Str fname;                                                                                                     \
		Location loc2;                                                                                                 \
		if (lua_gettop(s) < 2)                                                                                         \
		{                                                                                                              \
			if (log_warn("Expected two arguments to _log_" #lvl " but %d have been given", lua_gettop(s)))             \
				luaL_error(s, "Warnings are fatal");                                                                   \
			return 0;                                                                                                  \
		}                                                                                                              \
                                                                                                                       \
		char* msg = (char*)lua_tostring(s, -1);                                                                        \
                                                                                                                       \
		Location* loc;                                                                                                 \
		bool pop_table_vals = false;                                                                                   \
		if (lua_isnil(s, -2))                                                                                          \
		{                                                                                                              \
			loc = &loc2;                                                                                               \
			make_strv(&fname, DEFAULT_LOC);                                                                            \
			make_location(loc, 1, 1, 1, 1, &fname, false);                                                             \
		}                                                                                                              \
		if (lua_isuserdata(s, -2))                                                                                     \
		{                                                                                                              \
			LuaPointer* locp = lua_touserdata(s, -2);                                                                  \
			if (locp->type != LOCATION)                                                                                \
				luaL_error(s, "Location pointer has been changed! Expected pointer of type %d but got one of type %d", \
					LOCATION, locp->type);                                                                             \
			loc = locp->data;                                                                                          \
		}                                                                                                              \
		else if (lua_istable(s, -2))                                                                                   \
		{                                                                                                              \
			loc			   = &loc2;                                                                                    \
			pop_table_vals = true;                                                                                     \
			lua_getfield(s, -2, "first_line");                                                                         \
			lua_getfield(s, -3, "first_column");                                                                       \
			lua_getfield(s, -4, "last_line");                                                                          \
			lua_getfield(s, -5, "last_column");                                                                        \
			lua_getfield(s, -6, "src_file");                                                                           \
			make_location(loc, lua_tointeger(s, -5), lua_tointeger(s, -4), lua_tointeger(s, -3), lua_tointeger(s, -2), \
				&fname, false);                                                                                        \
			char* rawfname = (char*)lua_tostring(s, -1);                                                               \
			make_strv(&fname, rawfname ? rawfname : DEFAULT_LOC);                                                      \
		}                                                                                                              \
		else                                                                                                           \
		{                                                                                                              \
			const char* msg = luaL_tolstring(s, -1, NULL);                                                             \
			return luaL_error(                                                                                         \
				s, "Location value is not nil, a userdata pointer or a table, failed while logging '%s'", msg);        \
		}                                                                                                              \
		err_handler(log_##lvl##_at(loc, "%s", msg));                                                                   \
                                                                                                                       \
		if (pop_table_vals)                                                                                            \
			lua_pop(s, 5);                                                                                             \
		return 0;                                                                                                      \
	}

#define EXT_LOG_FUNC(lvl, err_handler)                                                                                 \
	static int ext_log_##lvl(ExtensionState* s)                                                                        \
	{                                                                                                                  \
		if (lua_gettop(s) < 1)                                                                                         \
		{                                                                                                              \
			if (log_warn("Expected two arguments to _log_" #lvl " but %d have been given", lua_gettop(s)))             \
				luaL_error(s, "Warnings are fatal");                                                                   \
			return 0;                                                                                                  \
		}                                                                                                              \
                                                                                                                       \
		char* msg = (char*)lua_tostring(s, -1);                                                                        \
		err_handler(log_##lvl("%s", msg));                                                                             \
                                                                                                                       \
		return 0;                                                                                                      \
	}

EXT_LOG_FUNC(err, ID)
EXT_LOG_AT_FUNC(err, ID)
EXT_LOG_FUNC(warn, EXIT_ON_ERR)
EXT_LOG_AT_FUNC(warn, EXIT_ON_ERR)
EXT_LOG_FUNC(info, ID)
EXT_LOG_FUNC(debug, ID)
