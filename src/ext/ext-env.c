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

#define EM_CONFIG_FILE_NAME		      "em_config_file"
#define EM_EVAL_NODE_FUNC_NAME		  "eval"
#define EM_REQUIRE_RUNS_FUNC_NAME	  "requires_reiter"
#define LUA_POINTER_GC_METATABLE_RKEY "emblem_core_pointer"

const char* const lua_pointer_type_names[] = {
	[DOC_TREE_NODE] = "doc-tree node",
	[STYLER]		= "styler",
	[EXT_ENV]		= "extension environment",
	[MT_NAMES_LIST] = "mt-safe file-name list",
	[PARSED_ARGS]	= "parsed command-line arguments",
};

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

static int ext_dest_lua_pointer(ExtensionState* s);
static void set_globals(ExtensionEnv* e, ExtParams* params);
static void load_em_std_functions(ExtensionState* s);
static int load_libraries(ExtensionState* s, ExtParams* params);
static void load_library_set(ExtensionState* s, luaL_Reg* lib);
static int ext_require_rerun(ExtensionState* s);

LuaPointer* new_lua_pointer(ExtensionState* s, LuaPointerType type, void* data, bool destruction_permitted)
{
	LuaPointer* ret			   = lua_newuserdatauv(s, sizeof(LuaPointer), 1);
	ret->type				   = type;
	ret->data				   = data;
	ret->valid				   = true;
	ret->destruction_permitted = destruction_permitted; // TODO: check that the destruction_permitted is honoured.

	lua_getfield(s, LUA_REGISTRYINDEX, LUA_POINTER_GC_METATABLE_RKEY);
	lua_setmetatable(s, -2);

	return ret;
}

static int ext_dest_lua_pointer(ExtensionState* s)
{
	if (lua_gettop(s) < 1)
		luaL_error(s, "Expected one argument to the lua pointer finaliser function");
	if (!lua_isuserdata(s, -1))
		luaL_error(
			s, "Expected userdata value to finalise, instead got %s (%s)", luaL_typename(s, -1), lua_tostring(s, -1));

	LuaPointer* lp = lua_touserdata(s, -1);
	if (!lp->valid || !lp->destruction_permitted)
		return 0;

	// Destroy contents as necessary
	switch (lp->type)
	{
		case DOC_TREE_NODE:
			dest_free_doc_tree_node(lp->data, false, LUA_POINTER_DEREFERENCE);
			break;
		default:
			log_warn("No destructor for lua pointer of type %d", lp->type);
			break;
	}

	return 0;
}

int to_userdata_pointer(void** val, ExtensionState* s, int idx, LuaPointerType type)
{
	if (!lua_isuserdata(s, idx))
	{
		log_err("Expected userdata but got %s '%s'", luaL_typename(s, idx), luaL_tolstring(s, idx, NULL));
		return 1;
	}
	LuaPointer* ptr = lua_touserdata(s, idx);
	if (!ptr->valid)
	{
		log_err("Attempted to dereference an invalid lua pointer of type %s", lua_pointer_type_names[type]);
		return 1;
	}
	if (ptr->type != type)
	{
		log_err("Expected %s userdata (%d) but got %s userdata (%d)", lua_pointer_type_names[type], type,
			lua_pointer_type_names[ptr->type], ptr->type);
		return 1;
	}
	*val = ptr->data;
	return 0;
}

void release_pass_local_lua_pointers(ExtensionEnv* e) { lua_gc(e->state, LUA_GCCOLLECT); }

void invalidate_lua_pointer(LuaPointer* lp) { lp->valid = false; }

int make_ext_env(ExtensionEnv* ext, ExtParams* params)
{
	int rc;
	ext->state			   = luaL_newstate();
	ext->require_extra_run = true;
	ext->iter_num		   = 0;
	log_debug("Getting created ext state at %p in env %p", (void*)ext->state, (void*)ext);

	set_globals(ext, params);

	load_arguments(ext, params->ext_args);

	log_info("Loading standard library...");
	if ((rc = load_libraries(ext->state, params)))
		return rc;

	return load_extensions(ext->state, params);
}

void dest_ext_env(ExtensionEnv* ext) { lua_close(ext->state); }

static void set_globals(ExtensionEnv* e, ExtParams* params)
{
	ExtensionState* s = e->state;

	ext_set_global_constants(s);

	// Garbage collector metatable for luapointers
	luaL_newmetatable(s, LUA_POINTER_GC_METATABLE_RKEY);
	/* lua_createtable(s, 0, 1); */
	lua_pushcfunction(s, ext_dest_lua_pointer);
	lua_setfield(s, -2, "__gc");
	/* lua_setfield(s, LUA_REGISTRYINDEX, LUA_POINTER_GC_METATABLE_RKEY); */
	lua_pop(s, 1);

	/* // Garbage collection preventors for lua pointers */
	/* lua_newtable(s); */
	/* [> lua_setfield(s, LUA_REGISTRYINDEX, LUA_POINTER_GC_PREVENTOR_GLOBAL); <] */
	/* lua_newtable(s); */
	/* lua_setfield(s, LUA_REGISTRYINDEX, LUA_POINTER_GC_PREVENTOR_PASS_LOCAL); */

	// Store the iteration number
	lua_pushinteger(s, 0);
	lua_setglobal(s, EM_ITER_NUM_VAR_NAME);

	// Allow the environment to access itself
	new_lua_pointer(s, EXT_ENV, e, false);
	lua_setglobal(s, EM_ENV_VAR_NAME);

	// Store the args in raw form
	new_lua_pointer(s, PARSED_ARGS, params->args, false);
	lua_setglobal(s, EM_ARGS_VAR_NAME);

	// Store the names list
	new_lua_pointer(s, MT_NAMES_LIST, params->mt_names_list, false);
	lua_setglobal(s, EM_MT_NAMES_LIST_VAR_NAME);

	// Store the styler
	new_lua_pointer(s, STYLER, params->styler, false);
	lua_setglobal(e->state, STYLER_LP_LOC);

	// Store the config file
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
