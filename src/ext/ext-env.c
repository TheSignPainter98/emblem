#include "ext-env.h"

#include "doc-struct/ast.h"
#include "logs/logs.h"
#include "lua-ast-io.h"
#include "lua-lib-load.h"
#include "ext-loader.h"
#include "style.h"
#include <lauxlib.h>

#define EM_EVAL_NODE_FUNC_NAME	  "eval"
#define EM_REQUIRE_RUNS_FUNC_NAME "requires_reiter"
#define EM_ENV_VAR_NAME			  "_em_env"
#define EM_NODE_TYPES_TABLE		  "_node_types"

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

static void set_globals(ExtensionEnv* e);
static void load_em_std_functions(ExtensionState* s);
static int load_libraries(ExtensionState* s, ExtParams* params);
static void load_library_set(ExtensionState* s, luaL_Reg* lib);
static int ext_require_rerun(ExtensionState* s);

int make_ext_env(ExtensionEnv* ext, ExtParams* params)
{
	ext->state			   = luaL_newstate();
	ext->require_extra_run = true;
	ext->iter_num		   = 0;
	ext->styler			   = malloc(sizeof(LuaPointer));
	make_lua_pointer(ext->styler, STYLER, params->styler);
	log_debug("Getting created ext state at %p in env %p", (void*)ext->state, (void*)ext);
	provide_styler(ext);

	set_globals(ext);

	int rc = load_libraries(ext->state, params);
	if (rc)
		return rc;

	return load_extensions(ext->state, params);
}

void dest_ext_env(ExtensionEnv* ext)
{
	lua_close(ext->state);
	dest_lua_pointer(ext->selfp, NULL);
	free(ext->selfp);
	dest_lua_pointer(ext->styler, NULL);
	free(ext->styler);
}

void finalise_env_for_typesetting(ExtensionEnv* e) { rescind_styler(e); }

static void set_globals(ExtensionEnv* e)
{
	ExtensionState* s = e->state;

	// Create the `em` table
	lua_newtable(s);
	lua_setglobal(s, EM_PUBLIC_TABLE);

	// Store the iteration number
	lua_pushinteger(s, 0);
	lua_setglobal(s, EM_ITER_NUM_VAR_NAME);

	// Allow the environment to access itself
	e->selfp = malloc(sizeof(LuaPointer));
	make_lua_pointer(e->selfp, EXT_ENV, e);
	lua_pushlightuserdata(s, e->selfp);
	lua_setglobal(s, EM_ENV_VAR_NAME);

	// Allow pretty names for the node types
	lua_newtable(s);
	for (size_t i = 0; i < node_tree_content_type_names_len; i++)
	{
		lua_pushinteger(s, i);
		lua_setfield(s, -2, node_tree_content_type_names[i]);
	}
	lua_setglobal(s, EM_NODE_TYPES_TABLE);
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
	lua_register(s, EM_IMPORT_STYLESHEET_FUNC_NAME, ext_import_stylesheet);
	lua_register(s, EM_REQUIRE_RUNS_FUNC_NAME, ext_require_rerun);
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
	if (!lua_isuserdata(s, -1))
		luaL_error(s,
			"Environment variable %s is not a userdata object (it is a %s value). There is no reason to change its "
			"value so please don't",
			EM_ENV_VAR_NAME, luaL_typename(s, -1));

	LuaPointer* lp = lua_touserdata(s, -1);
	if (lp->type != EXT_ENV)
		luaL_error(s,
			"Environment variable %s has been changed and no longer represents an environment. THere is no reason to "
			"change its value, so please don't",
			EM_ENV_VAR_NAME);
	lua_pop(s, -1);

	ExtensionEnv* e		 = lp->data;
	e->require_extra_run = true;
	return 0;
}
