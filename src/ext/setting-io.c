#include "setting-io.h"

#include "logs/logs.h"
#include <lauxlib.h>
#include <lua.h>

#define SETTING_GETTER_FUNC_NAME "get_conf"
#define SETTING_SETTER_FUNC_NAME "set_conf"
#define USED_SETTINGS_RIDX		 "emblem_used_settings"

/* static int ext_set_setting(ExtensionState* s); */
/* static int ext_get_setting(ExtensionState* s); */

void set_ext_setting_globals(ExtensionState* s)
{
	lua_newtable(s);
	lua_setfield(s, LUA_REGISTRYINDEX, USED_SETTINGS_RIDX);

	/* lua_register(s, EM_SET_SETTING_FUNC_NAME, ext_set_setting); */
	/* lua_register(s, EM_GET_SETTING_FUNC_NAME, ext_get_setting); */
}

int set_setting(ExtensionEnv* env, char* name, char* value)
{
	ExtensionState* s = env->state;
	lua_getglobal(s, SETTING_SETTER_FUNC_NAME);
	lua_pushstring(s, name);
	lua_pushstring(s, value);
	if (lua_pcall(s, 2, 0, 0) != LUA_OK)
	{
		log_warn("Problem setting setting '%s': %s", name, lua_tostring(s, -1));
		return 1;
	}
	return 0;
}

const char* get_setting(ExtensionEnv* env, const char* name)
{
	ExtensionState* s = env->state;

	// Get the setting
	lua_getglobal(s, SETTING_GETTER_FUNC_NAME);
	lua_pushstring(s, name);
	if (lua_pcall(s, 1, 1, 0) != LUA_OK)
	{
		log_warn("Problem getting setting '%s': %s", name, lua_tostring(s, -1));
		return NULL;
	}
	const char* ret = lua_tostring(s, -1);

	// Stymie gc
	lua_getfield(s, LUA_REGISTRYINDEX, USED_SETTINGS_RIDX);
	lua_len(s, -1);
	lua_pushnil(s);
	lua_copy(s, -4, -1);
	lua_settable(s, -3);
	lua_pop(s, 2);

	return ret;
}

void release_setting(ExtensionEnv* env)
{
	// Destyme gc
	ExtensionState* s = env->state;
	lua_getfield(s, LUA_REGISTRYINDEX, USED_SETTINGS_RIDX);
	lua_len(s, -1);
	lua_pushnil(s);
	lua_settable(s, -3);
	lua_pop(s, 1);
}
