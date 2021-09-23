/**
 * @file lua-events.c
 * @brief Implements callers for typesetting events for extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "lua-events.h"

#include "logs/logs.h"
#include <lauxlib.h>
#include <lualib.h>

#define ON_START_EVENT_NAME		 "on_start"
#define ON_ITER_START_EVENT_NAME "on_iter_start"
#define ON_ITER_END_EVENT_NAME	 "on_iter_end"
#define ON_END_EVENT_NAME		 "on_end"

static int do_event(ExtensionState* s, const char* event_name);

int do_lua_start_event(ExtensionState* s) { return do_event(s, ON_START_EVENT_NAME); }

int do_lua_iter_start_event(ExtensionState* s) { return do_event(s, ON_ITER_START_EVENT_NAME); }

int do_lua_iter_end_event(ExtensionState* s) { return do_event(s, ON_ITER_END_EVENT_NAME); }

int do_lua_end_event(ExtensionState* s) { return do_event(s, ON_END_EVENT_NAME); }

static int do_event(ExtensionState* s, const char* event_name)
{
	log_debug("Executing event '%s'", event_name);
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
