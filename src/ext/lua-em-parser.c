#include "lua-em-parser.h"

#include "argp.h"
#include "data/list.h"
#include "data/locked.h"
#include "data/maybe.h"
#include "data/str.h"
#include "debug.h"
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
	Args* args;
	int rc = to_userdata_pointer((void**)&args, s, -1, PARSED_ARGS);
	if (rc)
		luaL_error(s, "Invalid argument(s)");
	lua_getglobal(s, EM_MT_NAMES_LIST_VAR_NAME);

	Locked* mtNamesList;
	rc = to_userdata_pointer((void**)&mtNamesList, s, -1, MT_NAMES_LIST);
	if (rc)
		luaL_error(s, "Invalid argument(s)");
	lua_pop(s, 2);

	// Parse the file
	Maybe mpf;
	unsigned int nerrs = parse_file(&mpf, mtNamesList, args, fname);
	if (mpf.type == NOTHING)
		luaL_error(s, "Parsing %s failed with %d errors", fname, nerrs);
	DocTreeNode* included_root = mpf.just;
	dest_maybe(&mpf, NULL);

	lua_getglobal(s, EM_ENV_VAR_NAME);
	ExtensionEnv* env;
	rc = to_userdata_pointer((void**)&env, s, -1, EXT_ENV);
	if (rc)
		luaL_error(s, "Invalid internal value");
	lua_pop(s, 1);

	if (exec_lua_pass_on_node(s, included_root, env->iter_num))
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
