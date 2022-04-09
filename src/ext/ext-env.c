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

#define EM_CONFIG_FILE_NAME			  "em_config_file"
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
static void setup_api_table(ExtensionEnv* e, ExtParams* params);
static void load_em_std_apis(ExtensionState* s);
static int load_libraries(ExtensionState* s, ExtParams* params);
static void resist_api_table_changes(ExtensionState* s);
static int ext_api_table_reject_new_index(ExtensionState* s);
static void load_library_set(ExtensionState* s, luaL_Reg* lib);
static int ext_require_rerun(ExtensionState* s);

LuaPointer* new_lua_pointer(ExtensionState* s, LuaPointerType type, void* data)
{
	LuaPointer* ret			   = lua_newuserdatauv(s, sizeof(LuaPointer), 1);
	ret->type				   = type;
	ret->data				   = data;

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

	// Destroy contents as necessary
	LuaPointer* lp = lua_touserdata(s, -1);
	switch (lp->type)
	{
		case DOC_TREE_NODE:
			dest_free_doc_tree_node(lp->data, false, LUA_POINTER_DEREFERENCE);
			break;
		case LOCATION:
			dest_free_location(lp->data, LUA_POINTER_DEREFERENCE);
			break;
		default:
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

int make_ext_env(ExtensionEnv* ext, ExtParams* params)
{
	int rc;
	ext->state			   = luaL_newstate();
	ext->require_extra_run = true;
	ext->iter_num		   = 0;
	log_debug("Getting created ext state at %p in env %p", (void*)ext->state, (void*)ext);

	setup_api_table(ext, params);

	load_arguments(ext, params->ext_args);

	log_info("Loading standard library...");
	if ((rc = load_libraries(ext->state, params)))
		return rc;

	resist_api_table_changes(ext->state);

	return load_extensions(ext->state, params);
}

void dest_ext_env(ExtensionEnv* ext) { lua_close(ext->state); }

static void setup_api_table(ExtensionEnv* e, ExtParams* params)
{
	ExtensionState* s = e->state;

	// Store the Emblem API table
	lua_newtable(s);
	lua_pushstring(s, params->config_file->str);
	lua_setfield(s, -2, EM_CONFIG_FILE_NAME);

	setup_lua_constants_api(s);

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
	lua_setfield(s, -2, EM_ITER_NUM_VAR_NAME);

	// Allow the environment to access itself
	new_lua_pointer(s, EXT_ENV, e);
	lua_setfield(s, -2, EM_ENV_VAR_NAME);

	// Store the args in raw form
	new_lua_pointer(s, PARSED_ARGS, params->args);
	lua_setfield(s, -2, EM_ARGS_VAR_NAME);

	// Store the names list
	new_lua_pointer(s, MT_NAMES_LIST, params->mt_names_list);
	lua_setfield(s, -2, EM_MT_NAMES_LIST_VAR_NAME);

	// Store the styler
	new_lua_pointer(s, STYLER, params->styler);
	lua_setfield(s, -2, EM_STYLER_LP_LOC);

	lua_setglobal(s, EM_API_TABLE_NAME);
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

	load_em_std_apis(s);

	return load_em_std_lib(s);
}

static void load_em_std_apis(ExtensionState* s)
{
	lua_getglobal(s, EM_API_TABLE_NAME);
	register_api_function(s, EM_REQUIRE_RUNS_FUNC_NAME, ext_require_rerun);
	register_api_function(s, EM_INCLUDE_FILE_FUNC_NAME, ext_include_file);

	register_ext_logging(s);
	register_ext_location(s);
	register_ext_style(s);
	register_ext_setting(s);
	register_ext_node(s);
	lua_pop(s, 1);
}

void get_api_elem(ExtensionState* s, const char* name)
{
	lua_getglobal(s, EM_API_TABLE_NAME);
	lua_getfield(s, -1, name);
	lua_rotate(s, -2, 1);
	lua_pop(s, 1);
}

void set_api_elem(ExtensionState* s, int idx, const char* name)
{
	lua_getglobal(s, EM_API_TABLE_NAME);
	lua_rotate(s, idx - 1, -1);
	lua_setfield(s, -2, name);
	lua_pop(s, 1);
}

void update_api_elem(ExtensionState* s, int idx, const char* name)
{
	lua_getglobal(s, EM_API_TABLE_NAME);
	lua_pushstring(s, name);
	lua_rotate(s, idx - 2, -1);
	lua_rawset(s, -3);
	lua_pop(s, 1);
}

static void resist_api_table_changes(ExtensionState* s)
{
	lua_createtable(s, 0, 0);
	luaL_newmetatable(s, EM_API_TABLE_NAME);
	lua_getglobal(s, EM_API_TABLE_NAME);
	lua_setfield(s, -2, "__index");
	lua_pushcfunction(s, ext_api_table_reject_new_index);
	lua_setfield(s, -2, "__newindex");
	lua_setmetatable(s, -2);
	lua_setglobal(s, EM_API_TABLE_NAME);
}

static int ext_api_table_reject_new_index(ExtensionState* s)
{
	luaL_error(s, "The emblem API table should not be modified");
	return 0;
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

	get_api_elem(s, EM_ENV_VAR_NAME);
	ExtensionEnv* e;
	int rc = to_userdata_pointer((void**)&e, s, -1, EXT_ENV);
	lua_pop(s, 1);
	if (rc)
		luaL_error(s, "Invalid internal value");

	e->require_extra_run = true;
	return 0;
}
