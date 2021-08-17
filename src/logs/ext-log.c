#include "ext-log.h"

#include "ext/ext-env.h"
#include "logs.h"
#include <lauxlib.h>
#include <lua.h>

static int ext_log_err(ExtensionState* s);
static int ext_log_err_at(ExtensionState* s);
static int ext_log_warn(ExtensionState* s);
static int ext_log_warn_at(ExtensionState* s);
static int ext_log_info(ExtensionState* s);
static int ext_log_debug(ExtensionState* s);

void set_ext_logging_globals(ExtensionState* s)
{
	lua_register(s, "_log_err", ext_log_err);
	lua_register(s, "_log_err_at", ext_log_err_at);
	lua_register(s, "_log_warn", ext_log_warn);
	lua_register(s, "_log_warn_at", ext_log_warn_at);
	lua_register(s, "_log_info", ext_log_info);
	lua_register(s, "_log_debug", ext_log_debug);
}

#define ID(x) x
#define EXIT_ON_ERR(x)                                                                                                 \
	if (x)                                                                                                             \
		luaL_error(s, "Warnings are fatal");
#define EXT_LOG_AT_FUNC(lvl, err_handler)                                                                              \
	static int ext_log_##lvl##_at(ExtensionState* s)                                                                   \
	{                                                                                                                  \
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
			pop_table_vals = true;                                                                                     \
			Location loc2;                                                                                             \
			Str fname;                                                                                                 \
			lua_getfield(s, -2, "first_line");                                                                         \
			lua_getfield(s, -3, "first_column");                                                                       \
			lua_getfield(s, -4, "last_line");                                                                          \
			lua_getfield(s, -5, "last_column");                                                                        \
			lua_getfield(s, -6, "src_file");                                                                           \
			loc2.first_line	  = lua_tointeger(s, -5);                                                                  \
			loc2.first_column = lua_tointeger(s, -4);                                                                  \
			loc2.last_line	  = lua_tointeger(s, -3);                                                                  \
			loc2.last_column  = lua_tointeger(s, -2);                                                                  \
			loc2.src_file	  = &fname;                                                                                \
			char* rawfname	  = (char*)lua_tostring(s, -1);                                                            \
			make_strv(&fname, rawfname ? rawfname : "(UNSPECIFIED LOCATION)");                                         \
			loc = &loc2;                                                                                               \
		}                                                                                                              \
		else                                                                                                           \
		{                                                                                                              \
			luaL_error(s, "Location value is not a userdata pointer or a table");                                      \
			return 0; /* never happens */                                                                              \
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
