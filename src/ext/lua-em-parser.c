#include "lua-em-parser.h"

#include "argp.h"
#include "data/list.h"
#include "data/locked.h"
#include "data/maybe.h"
#include "data/str.h"
#include "logs/logs.h"
#include "lua-ast-io.h"
#include "lua-pointers.h"
#include "lua.h"
#include "parser/emblem-parser.h"
#include <lauxlib.h>
#include <stdlib.h>

int ext_include_file(ExtensionState* s)
{
	if (lua_gettop(s) != 1)
		if (log_warn(
				"Expected exactly one argument to " EM_INCLUDE_FILE_FUNC_NAME " but %d have been given", lua_gettop(s)))
			luaL_error(s, "Warnings are fatal");
	char* fname = (char*)lua_tostring(s, -1);
	lua_pop(s, 1);

	// Get the arguments and file names list
	lua_getglobal(s, EM_ARGS_VAR_NAME);
	if (!lua_islightuserdata(s, -1))
		luaL_error(s, "Variable " EM_ARGS_VAR_NAME " has been changed illegally and can no longer be used");
	LuaPointer* lpa = lua_touserdata(s, -1);
	if (lpa->type != PARSED_ARGS)
		luaL_error(s,
			"Variable " EM_ARGS_VAR_NAME
			" has been changed, expected pointer to object of type %d but got one to an object of type %d instead",
			PARSED_ARGS, lpa->type);
	Args* args = lpa->data;
	lua_getglobal(s, EM_MT_NAMES_LIST_VAR_NAME);
	if (!lua_islightuserdata(s, -1))
		luaL_error(s, "Variable " EM_MT_NAMES_LIST_VAR_NAME " has been changed illegally and can no longer be used");
	LuaPointer* lpmnl = lua_touserdata(s, -1);
	if (lpmnl->type != MT_NAMES_LIST)
		luaL_error(s,
			"Variable " EM_MT_NAMES_LIST_VAR_NAME
			" has been changed, expected a pointer to an object of type %d but got one to an object of type %d instead",
			MT_NAMES_LIST, lpmnl->type);
	Locked* mtNamesList = lpmnl->data;
	lua_pop(s, 2);

	// Parse the file
	Maybe mpf;
	int nerrs = parse_file(&mpf, mtNamesList, args, fname);
	if (mpf.type == NOTHING)
		luaL_error(s, "Parsing %s failed with %d errors", fname, nerrs);
	DocTreeNode* included_root = mpf.just;
	dest_maybe(&mpf, NULL);

	lua_getglobal(s, EM_ENV_VAR_NAME);
	ExtensionEnv* env;
	if (to_userdata_pointer((void**)&env, s, -1, EXT_ENV))
		return 1;
	lua_pop(s, 1);

	if(exec_lua_pass_on_node(s, included_root, env->iter_num))
		lua_pushnil(s);
	else
	{
		pack_tree(s, included_root); // TODO: maybe pushlightuserdata here to a pointer to the result, not the result itself?
		/* LuaPointer* lp = malloc(sizeof(LuaPointer)); */
		/* make_lua_pointer(lp, AST_NODE, included_root); */
		/* lua_pushlightuserdata(s, lp); */
		/* log_warn("================================================================================"); */
	}

	return 1;
}
