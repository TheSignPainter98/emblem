#include "lua-events.h"

#include "logs/logs.h"
#include <lauxlib.h>

#define ON_START_EVENT_NAME		 "on_start"
#define ON_ITER_START_EVENT_NAME "on_iter_start"
#define ON_ITER_END_EVENT_NAME	 "on_iter_end"
#define ON_END_EVENT_NAME		 "on_end"

static int do_event(ExtensionState* s, const char* event_name);

/*
 * #define HANDLE_LUA_EXTREMAL_EVENT(name, L) \
 *
 * // TODO: Remove this old way of handling reruns! Simplify!
 * #define HANDLE_LUA_ITER_EVENT(L, name, iter_num) \
 *     lua_getglobal(L, name); \
 *     int rc = lua_pcall(L, 0, 0, 0); \
 *     switch (rc) \
 *     { \
 *         case LUA_OK: \
 *             return 0; \
 *         case LUA_YIELD: \
 *             log_warn("Extension " name " function yielded instead of returned"); \
 *             return 1; \
 *         default: \
 *             log_err("Running " name " event failed with error: %s", lua_tostring(L, -1)); \
 *             return 1; \
 *     }
 */

int do_lua_start_event(ExtensionState* s) { return do_event(s, ON_START_EVENT_NAME); }

int do_lua_iter_start_event(ExtensionState* s) { return do_event(s, ON_ITER_START_EVENT_NAME); }

int do_lua_iter_end_event(ExtensionState* s) { return do_event(s, ON_ITER_END_EVENT_NAME); }

int do_lua_end_event(ExtensionState* s) { return do_event(s, ON_END_EVENT_NAME); }

static int do_event(ExtensionState* s, const char* event_name)
{
	lua_getglobal(s, event_name);
	int rc = lua_pcall(s, 0, 0, 0);
	switch (rc)
	{
		case LUA_OK:
			return 0;
		case LUA_YIELD:
			if (log_warn("Running %s event yielded instead of returned", event_name))
				luaL_error(s, "Warnings are fatal");
			return 1;
		default:
			log_err("Running %s event failed with error: %s", event_name, lua_tostring(s, -1));
			return 1;
	}
}
