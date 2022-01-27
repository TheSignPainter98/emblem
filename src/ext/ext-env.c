/**
 * @file ext-env.c
 * @brief Implements the Lua extension environment, loading libraries, extensions and pointers
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "ext-env.h"

#include "doc-struct/ast.h"
#include "doc-struct/location.h"
#include "ext-loader.h"
#include "logs/ext-log.h"
#include "logs/logs.h"
#include "lua-ast-io.h"
#include "lua-constants.h"
#include "lua-em-parser.h"
#include "lua-lib-load.h"
#include "lua.h"
#include "setting-io.h"
#include "style.h"
#include <lauxlib.h>

#define EM_EVAL_NODE_FUNC_NAME	  "eval"
#define EM_REQUIRE_RUNS_FUNC_NAME "requires_reiter"
#define EM_CONFIG_FILE_NAME		  "em_config_file"

static luaL_Reg lua_std_libs_universal[] = {
	{ "", luaopen_base },
	{ LUA_LOADLIBNAME, luaopen_package },
	{ LUA_COLIBNAME, luaopen_coroutine },
	{ LUA_UTF8LIBNAME, luaopen_utf8 },
	{ LUA_TABLIBNAME, luaopen_table },
	{ LUA_STRLIBNAME, luaopen_string },
	{ LUA_MATHLIBNAME, luaopen_math },
	{ LUA_DBLIBNAME, luaopen_debug },
	{ NULL, NULL },
};

static luaL_Reg lua_std_libs_restriction_lvl_1[] = {
	{ LUA_IOLIBNAME, luaopen_io },
	{ NULL, NULL },
};

static luaL_Reg lua_std_libs_restriction_lvl_0[] = {
	{ LUA_OSLIBNAME, luaopen_os },
	{ NULL, NULL },
};

static void set_globals(ExtensionEnv* e, ExtParams* params);
static void load_em_std_functions(ExtensionState* s);
static int load_libraries(ExtensionState* s, ExtParams* params);
static void load_library_set(ExtensionState* s, luaL_Reg* lib);
static int ext_require_rerun(ExtensionState* s);

int make_ext_env(ExtensionEnv* ext, ExtParams* params)
{
	int rc;
	ext->state			   = luaL_newstate();
	ext->require_extra_run = true;
	ext->iter_num		   = 0;
	ext->styler			   = malloc(sizeof(LuaPointer));
	make_lua_pointer(ext->styler, STYLER, params->styler);
	log_debug("Getting created ext state at %p in env %p", (void*)ext->state, (void*)ext);
	provide_styler(ext);

	set_globals(ext, params);

	load_arguments(ext, params->ext_args);

	log_info("Loading standard library...");
	if ((rc = load_libraries(ext->state, params)))
		return rc;

	return load_extensions(ext->state, params);
}

void dest_ext_env(ExtensionEnv* ext)
{
	lua_close(ext->state);
	dest_lua_pointer(ext->mt_names_list, NULL);
	free(ext->mt_names_list);
	dest_lua_pointer(ext->args, NULL);
	free(ext->args);
	dest_lua_pointer(ext->selfp, NULL);
	free(ext->selfp);
	dest_lua_pointer(ext->styler, NULL);
	free(ext->styler);
}

static void set_globals(ExtensionEnv* e, ExtParams* params)
{
	ExtensionState* s = e->state;

	ext_set_global_constants(s);

	// Store the iteration number
	lua_pushinteger(s, 0);
	lua_setglobal(s, EM_ITER_NUM_VAR_NAME);

	// Allow the environment to access itself
	e->selfp = malloc(sizeof(LuaPointer));
	make_lua_pointer(e->selfp, EXT_ENV, e);
	lua_pushlightuserdata(s, e->selfp);
	lua_setglobal(s, EM_ENV_VAR_NAME);

	// Store the args in raw form
	e->args = malloc(sizeof(LuaPointer));
	make_lua_pointer(e->args, PARSED_ARGS, params->args);
	lua_pushlightuserdata(s, e->args);
	lua_setglobal(s, EM_ARGS_VAR_NAME);

	// Store the names list
	e->mt_names_list = malloc(sizeof(LuaPointer));
	make_lua_pointer(e->mt_names_list, MT_NAMES_LIST, params->mt_names_list);
	lua_pushlightuserdata(s, e->mt_names_list);
	lua_setglobal(s, EM_MT_NAMES_LIST_VAR_NAME);

	lua_pushstring(s, params->config_file->str);
	lua_setglobal(s, EM_CONFIG_FILE_NAME);
}

#define LOAD_LIBRARY_SET(lvl, s, lib)                                                                                  \
	if (params->sandbox_lvl <= (lvl))                                                                                  \
	{                                                                                                                  \
		load_library_set(s, lib);                                                                                      \
	}

static int load_libraries(ExtensionState* s, ExtParams* params)
{
	LOAD_LIBRARY_SET(2, s, lua_std_libs_universal);
	LOAD_LIBRARY_SET(1, s, lua_std_libs_restriction_lvl_1);
	LOAD_LIBRARY_SET(0, s, lua_std_libs_restriction_lvl_0);

	load_em_std_functions(s);

	return load_em_std_lib(s);
}

static void load_em_std_functions(ExtensionState* s)
{
	lua_register(s, EM_EVAL_NODE_FUNC_NAME, ext_eval_tree);
	lua_register(s, EM_REQUIRE_RUNS_FUNC_NAME, ext_require_rerun);
	lua_register(s, EM_INCLUDE_FILE_FUNC_NAME, ext_include_file);

	set_ext_logging_globals(s);
	set_ext_location_globals(s);
	set_ext_style_globals(s);
	set_ext_setting_globals(s);
}

static void load_library_set(ExtensionState* s, luaL_Reg* lib)
{
	while (lib->func)
	{
		luaL_requiref(s, lib->name, lib->func, 1);
		lua_pop(s, 1); // remove lib
		lib++;
	}
}

static int ext_require_rerun(ExtensionState* s)
{
	if (lua_gettop(s) != 0)
		if (log_warn("Arguments to %s are ignored", EM_REQUIRE_RUNS_FUNC_NAME))
			luaL_error(s, "Warnings are fatal");

	lua_getglobal(s, EM_ENV_VAR_NAME);
	ExtensionEnv* e;
	int rc = to_userdata_pointer((void**)&e, s, -1, EXT_ENV);
	lua_pop(s, 1);
	if (rc)
		luaL_error(s, "Invalid internal value");

	e->require_extra_run = true;
	return 0;
}
