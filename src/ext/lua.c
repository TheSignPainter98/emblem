#include "lua.h"

#include "logs/logs.h"
#include "lua-ast-io.h"
#include "style.h"
#include <lauxlib.h>
#include <lualib.h>
#include <stdbool.h>
#include <string.h>

#define CLOSE_VAR_SCOPE_FUNC_NAME "close_var_scope"
#define OPEN_VAR_SCOPE_FUNC_NAME  "open_var_scope"
#define SET_VAR_FUNC_NAME		  "set-var"

static bool is_callable(ExtensionState* s, int idx);

void inc_iter_num(Doc* doc)
{
	doc->ext->iter_num++;
	lua_pushinteger(doc->ext->state, doc->ext->iter_num);
	lua_setglobal(doc->ext->state, EM_ITER_NUM_VAR_NAME);
}

int exec_lua_pass(Doc* doc)
{
	int rc = exec_lua_pass_on_node(doc->ext->state, doc->root, doc->ext->iter_num);
	lua_settop(doc->ext->state, 0);
	return rc;
}

#include "debug.h"
int exec_lua_pass_on_node(ExtensionState* s, DocTreeNode* node, int curr_iter)
{
	// Takes a node at the top of the stack, replaces it with the result of executing a lua pass
	if (!node)
		return 0;
	if (node->last_eval >= curr_iter)
		return 0;
	node->last_eval = curr_iter;
	switch (node->content->type)
	{
		case WORD:
			return 0;
		case CALL:
		{
			// Remove old result if present
			if (node->content->call->result)
				dest_free_doc_tree_node(node->content->call->result, true);

			lua_getglobal(s, EM_PUBLIC_TABLE);
			lua_getfield(s, -1, node->name->str);
			if (lua_isnoneornil(s, -1))
			{
				lua_pop(s, 2); // Remove nil value and public table
				node->flags |= CALL_HAS_NO_EXT_FUNC;
				if (is_empty_list(node->content->call->args))
				{
					int rc = log_warn_at(node->src_loc,
						"Directive '.%s' is not an extension function and has no arguments (would style nothing)",
						node->name->str);
					return rc ? -1 : 0;
				}
				if (node->content->call->args->cnt == 1)
				{
					// Pass through non-extension calls with a single argument
					node->content->call->result = node->content->call->args->fst->data;
					return exec_lua_pass_on_node(s, node->content->call->result, curr_iter);
				}

				log_debug("Putting args into lines node for result");
				DocTreeNode* resultNode = malloc(sizeof(DocTreeNode));
				make_doc_tree_node_content(resultNode, dup_loc(node->src_loc));
				resultNode->flags |= IS_GENERATED_NODE;
				ListIter li;
				make_list_iter(&li, node->content->call->args);
				DocTreeNode* currArg;
				while (iter_list((void**)&currArg, &li))
					prepend_doc_tree_node_child(resultNode, resultNode->content->content, currArg);
				node->content->call->result = resultNode;

				return exec_lua_pass_on_node(s, node->content->call->result, curr_iter);
			}
			if (!is_callable(s, -1))
			{
				lua_pop(s, 2); // Remove non-callable object and public table
				log_err("Expected function or callable table at em.%s, but got a %s", node->name->str,
					luaL_typename(s, -1));
				node->flags |= CALL_HAS_NO_EXT_FUNC;
				return -1;
			}

			// Prepare arguments
			const size_t num_args = node->content->call->args->cnt;
			ListIter li;
			make_list_iter(&li, node->content->call->args);
			DocTreeNode* argNode;
			LuaPointer argPtrs[num_args];
			int i = 0;
			while (iter_list((void**)&argNode, &li))
			{
				make_lua_pointer(&argPtrs[i], AST_NODE, argNode);
				lua_pushlightuserdata(s, &argPtrs[i]);
				i++;
			}
			dest_list_iter(&li);

			// Open variable scope
			lua_getglobal(s, OPEN_VAR_SCOPE_FUNC_NAME);
			if (!is_callable(s, -1))
			{
				log_err("Variable " OPEN_VAR_SCOPE_FUNC_NAME " is not a callable. Something has changed this!");
				return -1;
			}
			if (lua_pcall(s, 0, 0, 0) != LUA_OK)
			{
				log_err("Failed to open new variable scope: %s", lua_tostring(s, -1));
				return -1;
			}

			/* // Load the arguments */
			/* dumpstack(s); */
			/* lua_getglobal(s, EM_PUBLIC_TABLE); */
			/* for (int i = 0; i < num_args; i++) */
			/* { */
				/* log_info("Pushing arg..."); */
				/* lua_getfield(s, -1, SET_VAR_FUNC_NAME); */
				/* lua_pushinteger(s, i + 1); */
				/* lua_pushlightuserdata(s, &argPtrs[i]); */
				/* dumpstack(s); */
				/* switch (lua_pcall(s, 2, 0, 0)) */
				/* { */
					/* case LUA_OK: */
						/* break; */
					/* case LUA_YIELD: */
						/* if (log_warn_at( */
								/* node->src_loc, "Lua function " SET_VAR_FUNC_NAME " yielded instead of returned")) */
							/* return -1; */
						/* return 0; */
					/* default: */
						/* log_err_at( */
							/* node->src_loc, "Calling " SET_VAR_FUNC_NAME " failed with error: %s", lua_tostring(s, -1)); */
						/* return -1; */
				/* } */
			/* } */
			/* lua_pop(s, 1); */
			/* dumpstack(s); */

			// Call function
			log_debug("Pre-call stack:");
			dumpstack(s);
			log_debug("(Pcalling %s with %ld arguments...)", node->name->str, num_args);
			int rc;
			switch (lua_pcall(s, num_args, 1, 0))
			{
				case LUA_OK:
					log_debug("returned: %s", luaL_typename(s, -1));
					dumpstack(s);
					rc = unpack_lua_result(&node->content->call->result, s, node);
					log_debug("Unpacked result into %p", (void*)node->content->call->result);
					if (!rc)
						rc = exec_lua_pass_on_node(s, node->content->call->result, curr_iter);
					if (!rc)
						lua_pop(s, 1); // Pop the public table
					dumpstack(s);
					break;
				case LUA_YIELD:
				{
					int fw
						= log_warn_at(node->src_loc, "Lua function em.%s yielded instead of returned", node->name->str);
					lua_pop(s, 1); // Pop the public table
					rc = fw ? -1 : 0;
					break;
				}
				default:
					log_err_at(
						node->src_loc, "Calling em.%s failed with error: %s", node->name->str, lua_tostring(s, -1));
					lua_pop(s, 1); // Pop the public table
					rc = -1;
					break;
			}

			// Close variable scope
			lua_getglobal(s, CLOSE_VAR_SCOPE_FUNC_NAME);
			if (!is_callable(s, -1))
			{
				log_err("Variable " CLOSE_VAR_SCOPE_FUNC_NAME " is not a callable. Something has changed this!");
				return -1;
			}
			if (lua_pcall(s, 0, 0, 0) != LUA_OK)
			{
				log_err("Failed to open new variable scope");
				return -1;
			}
			return rc;
		}
		case CONTENT:
		{
			ListIter li;
			make_list_iter(&li, node->content->content);
			DocTreeNode* subNode;
			int rc = 0;
			while (iter_list((void**)&subNode, &li))
			{
				rc |= exec_lua_pass_on_node(s, subNode, curr_iter);
				if (rc)
					return rc;
			}
			dest_list_iter(&li);
			return rc;
		}
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

	lua_getfield(s, idx, "__call");
	bool callable = lua_isfunction(s, -1);
	lua_pop(s, 1);

	return callable;
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
		log_err("Expected %s userdata (%d) but got %s userdata (%d)", lua_pointer_type_names[type], type, lua_pointer_type_names[ptr->type], ptr->type);
		return 1;
	}
	*val = ptr->data;
	return 0;
}
