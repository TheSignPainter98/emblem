#include "lua.h"

#include "logs/logs.h"
#include "lua-ast-io.h"
#include "lua-lib-load.h"
#include "lua-pointers.h"
#include "style.h"
#include <lauxlib.h>
#include <lualib.h>
#include <string.h>

#define EM_EVAL_NODE_FUNC_NAME	   "eval"
#define EM_PUBLIC_TABLE			   "em"
#define EM_AST_NODE_TYPE_FUNC_NAME "ast_type_name"
#define EM_REQUIRE_RUNS_FUNC_NAME  "requires_reiter"
#define EM_ITER_NUM_VAR_NAME	   "em_iter"
#define EM_ENV_VAR_NAME 		   "_em_env"

static void load_em_std_functions(ExtensionState* s);
static int load_libraries(ExtensionState* s, ExtParams* params);
static void load_library_set(ExtensionState* s, luaL_Reg* lua_std_libs);
static bool is_callable(ExtensionState* s, int idx);
static int ext_require_rerun(ExtensionState* s);

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

int make_doc_ext_state(Doc* doc, ExtParams* params)
{
	doc->ext					= malloc(sizeof(ExtensionEnv));
	doc->ext->state				= luaL_newstate();
	doc->ext->require_extra_run = true;
	doc->ext->iter_num			= 0;
	doc->ext->styler			= malloc(sizeof(LuaPointer));
	make_lua_pointer(doc->ext->styler, STYLER, doc->styler);
	provide_styler(doc->ext);

	doc->ext->selfp = malloc(sizeof(LuaPointer));
	make_lua_pointer(doc->ext->selfp, EXT_ENV, doc->ext);
	lua_pushlightuserdata(doc->ext->state, doc->ext->selfp);
	lua_setglobal(doc->ext->state, EM_ENV_VAR_NAME);

	return load_libraries(doc->ext->state, params);
}

#define LOAD_LIBRARY_SET(lvl, s, lib)                                                                                  \
	if (params->sandbox_lvl <= lvl)                                                                                    \
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
	lua_register(s, EM_AST_NODE_TYPE_FUNC_NAME, get_ast_type_name);
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

void dest_doc_ext_state(Doc* doc)
{
	lua_close(doc->ext->state);
	dest_lua_pointer(doc->ext->selfp, NULL);
	free(doc->ext->selfp);
	dest_lua_pointer(doc->ext->styler, NULL);
	free(doc->ext->styler);
	free(doc->ext);
}

void inc_iter_num(ExtensionEnv* e)
{
	e->iter_num++;
	lua_pushinteger(e->state, e->iter_num);
	lua_setglobal(e->state, EM_ITER_NUM_VAR_NAME);
}

int exec_lua_pass(Doc* doc) { return exec_lua_pass_on_node(doc->ext->state, doc->root); }

#define HANDLE_LUA_LIST_PASS(s, list)                                                                                  \
	{                                                                                                                  \
		ListIter li;                                                                                                   \
		make_list_iter(&li, list);                                                                                     \
		DocTreeNode* subNode;                                                                                          \
		int rc = 0;                                                                                                    \
		while (iter_list((void**)&subNode, &li))                                                                       \
		{                                                                                                              \
			rc |= exec_lua_pass_on_node(s, subNode);                                                                   \
			if (rc)                                                                                                    \
				return rc;                                                                                             \
		}                                                                                                              \
		dest_list_iter(&li);                                                                                           \
		return rc;                                                                                                     \
	}

#include "debug.h"
int exec_lua_pass_on_node(ExtensionState* s, DocTreeNode* node)
{
	log_debug("-asdf-");
	dumpstack(s);
	log_debug("-asdf-");
	switch (node->content->type)
	{
		case WORD:
			return 0;
		case CALL:
		{
			lua_getglobal(s, EM_PUBLIC_TABLE);
			lua_getfield(s, -1, node->name->str);
			if (lua_isnoneornil(s, -1))
			{
				node->flags |= CALL_HAS_NO_EXT_FUNC;
				if (is_empty_list(node->content->call_params->args))
				{
					int rc = log_warn_at(node->src_loc, "Directive '.%s' is not an extension function and has no arguments (would style nothing)", node->name->str);
					lua_pop(s, -1); // Remove call function
					return rc ? -1 : 0;
				}
				else if (node->content->call_params->args->cnt == 1)
				{
					node->content->call_params->result = node->content->call_params->args->fst->data;
					lua_pop(s, -1); // Remove call function
					return 0;
				}
				else
				{
					log_debug("Putting args into lines node for result");
					DocTreeNode* linesNode = malloc(sizeof(DocTreeNode));
					make_doc_tree_node_lines(linesNode, dup_loc(node->src_loc));
					linesNode->flags |= IS_GENERATED_NODE;
					ListIter li;
					make_list_iter(&li, node->content->call_params->args);
					DocTreeNode* currArg;
					while (iter_list((void**)&currArg, &li))
						prepend_doc_tree_node_child(linesNode, linesNode->content->lines, currArg);
					node->content->call_params->result = linesNode;
					lua_pop(s, -1); // Remove call function
					return 0;
				}
			}
			else if (!is_callable(s, -1))
			{
				log_err("Expected function or callable table at em.%s", node->name->str);
				log_err("Got a %s", luaL_typename(s, -1));
				node->flags |= CALL_HAS_NO_EXT_FUNC;
				lua_pop(s, -1); // Remove call function
				return -1;
			}

			// Remove old result of present
			if (node->content->call_params->result)
				dest_free_doc_tree_node(node->content->call_params->result, true);

			// Prepare arguments
			const int num_args = node->content->call_params->args->cnt;
			ListIter li;
			make_list_iter(&li, node->content->call_params->args);
			DocTreeNode* argNode;
			LuaPointer argPtrs[num_args];
			int i = 0;
			while (iter_list((void**)&argNode, &li))
			{
				make_lua_pointer(&argPtrs[i], AST_NODE, argNode);
				lua_pushlightuserdata(s, &argPtrs[i]);
				i++;
			}

			log_debug("Stack:");
			dumpstack(s);
			log_debug("Calling %s...", node->name->str);
			log_debug("(Pcalling %s with %d arguments...)", node->name->str, num_args);
			switch (lua_pcall(s, num_args, 1, 0))
			{
				case LUA_OK:
					log_debug("returned: %s", luaL_typename(s, -1));
					return unpack_lua_result(&node->content->call_params->result, s, node);
				case LUA_YIELD:
				{
					int fw = log_warn_at(node->src_loc, "Lua function em.%s yielded instead of returned", node->name->str);
					return fw ? -1 : 0;
				}
				default:
					log_err_at(node->src_loc, "Calling em.%s failed with error: %s", node->name->str, lua_tostring(s, -1));
					return -1;
			}
		}
		case LINE:
			HANDLE_LUA_LIST_PASS(s, node->content->line);
		case LINES:
			HANDLE_LUA_LIST_PASS(s, node->content->lines);
		case PAR:
			HANDLE_LUA_LIST_PASS(s, node->content->par);
		case PARS:
			HANDLE_LUA_LIST_PASS(s, node->content->pars);
		default:
			log_err("Failed to perform lua pass, encountered node of unknown type %d", node->content->type);
			return -1;
	}
}

static bool is_callable(ExtensionState* s, int idx)
{
	if (lua_isfunction(s, idx))
		return true;

	if (!lua_istable(s, idx))
		return false;

	if (!lua_getmetatable(s, idx))
		return false;

	lua_getfield(s, -1, "__call");
	bool callable = lua_isfunction(s, -1);
	lua_pop(s, -1);

	return callable;
}

static int ext_require_rerun(ExtensionState* s)
{
	if (lua_gettop(s) != 0)
		if (log_warn("Arguments to %s are ignored", EM_REQUIRE_RUNS_FUNC_NAME))
			luaL_error(s, "Warnings are fatal");

	lua_getglobal(s, EM_ENV_VAR_NAME);
	if (!lua_isuserdata(s, -1))
		luaL_error(s, "Environment variable %s is not a userdata object (it is a %s value). There is no reason to change its value so please don't", EM_ENV_VAR_NAME, luaL_typename(s, -1));

	LuaPointer* lp = lua_touserdata(s, -1);
	if (lp->type != EXT_ENV)
		luaL_error(s, "Environment variable %s has been changed and no longer represents an environment. THere is no reason to change its value, so please don't", EM_ENV_VAR_NAME);
	lua_pop(s, -1);

	ExtensionEnv* e = lp->data;
	e->require_extra_run = true;
	return 0;
}
