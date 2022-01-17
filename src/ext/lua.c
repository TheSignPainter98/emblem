/**
 * @file lua.c
 * @brief Provides functions for executing evaluation-passes on document trees
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "lua.h"

#include "doc-struct/discern-pars.h"
#include "logs/logs.h"
#include "lua-ast-io.h"
#include "style.h"
#include "style/selection-engine.h"
#include <lauxlib.h>
#include <lualib.h>
#include <stdbool.h>
#include <string.h>

#define CLOSE_VAR_SCOPE_FUNC_NAME "close_var_scope"
#define OPEN_VAR_SCOPE_FUNC_NAME  "open_var_scope"

static bool is_callable(ExtensionState* s, int idx);
static int evaluate_directives(ExtensionState* s, DocTreeNode* node, int curr_iter) __attribute__((nonnull(1)));
static int resolve_styling(DocTreeNode* node, Styler* sty) __attribute__((nonnull(1, 2)));

void inc_iter_num(Doc* doc)
{
	doc->ext->iter_num++;
	lua_pushinteger(doc->ext->state, doc->ext->iter_num);
	lua_setglobal(doc->ext->state, EM_ITER_NUM_VAR_NAME);
}

int exec_lua_pass(Doc* doc)
{
	int rc
		= exec_lua_pass_on_node(doc->ext->state, doc->styler, doc->root, doc->ext->iter_num, doc->ext->iter_num == 1);
	lua_settop(doc->ext->state, 0);
	return rc;
}

#include "debug.h"
int exec_lua_pass_on_node(ExtensionState* s, Styler* sty, DocTreeNode* node, int curr_iter, bool foster_paragraphs)
{
	if (!node)
		return 0;
	node->last_eval = curr_iter;

	// Evaluate the directives contained
	int rc = evaluate_directives(s, node, curr_iter);
	if (rc)
		return rc;

	// Add foster paragraphs if and where necessary
	if (foster_paragraphs && (rc = introduce_foster_pars(node)))
		return rc;

	// Resolve the resulting styling
	return resolve_styling(node, sty);
}

static int evaluate_directives(ExtensionState* s, DocTreeNode* node, int curr_iter)
{
	int rc = 0;

	if (!node)
		return 0;
	if (node->flags & NO_FURTHER_EVAL)
		return 0;

	// Exit if no further evaluation is required.
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
			if (lua_isnoneornil(s, -1) || node->flags & STYLE_DIRECTIVE_ONLY)
			{
				lua_pop(s, 2); // Remove nil value and public table
				node->flags |= CALL_HAS_NO_EXT_FUNC;
				if (is_empty_list(node->content->call->args))
				{
					if (log_warn_at(node->src_loc,
							"Directive '.%s' is not an extension function and has no arguments (would style nothing)",
							node->name->str))
						return -1;
				}
				if (node->content->call->args->cnt == 1)
				{
					// Pass through non-extension calls with a single argument
					node->content->call->result = node->content->call->args->fst->data;
					return evaluate_directives(s, node->content->call->result, curr_iter);
				}

				log_debug("Putting args into lines node for result");
				DocTreeNode* resultNode = malloc(sizeof(DocTreeNode));
				make_doc_tree_node_content(resultNode, dup_loc(node->src_loc));
				resultNode->flags |= IS_GENERATED_NODE;
				ListIter li;
				make_list_iter(&li, node->content->call->args);
				DocTreeNode* currArg;
				while (iter_list((void**)&currArg, &li))
					append_doc_tree_node_child(resultNode, resultNode->content->content, currArg);
				node->content->call->result = resultNode;

				return evaluate_directives(s, node->content->call->result, curr_iter);
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

			// Update the location
			lua_getglobal(s, "set_var");
			lua_pushliteral(s, EM_LOC_NAME);
			LuaPointer llp;
			make_lua_pointer(&llp, LOCATION, node->src_loc);
			lua_pushlightuserdata(s, &llp);
			if (lua_pcall(s, 2, 0, 0) != LUA_OK)
			{
				log_err_at(node->src_loc, "Failed to set location information: %s", lua_tostring(s, -1));
				return -1;
			}

			// Call function
			log_debug("Pre-call stack:");
			dumpstack(s);
			log_debug("(Pcalling %s with %ld arguments...)", node->name->str, num_args);
			switch (lua_pcall(s, num_args, 1, 0))
			{
				case LUA_OK:
					log_debug("returned: %s", luaL_typename(s, -1));
					dumpstack(s);
					rc = unpack_lua_result(&node->content->call->result, s, node);
					log_debug("Unpacked result into %p", (void*)node->content->call->result);
					if (!rc)
						rc = evaluate_directives(s, node->content->call->result, curr_iter);
					if (!rc)
						lua_pop(s, 1); // Pop the public table
					dumpstack(s);
					break;
				case LUA_YIELD:
				{
					rc |= log_warn_at(node->src_loc, "Lua function em.%s yielded instead of returned", node->name->str);
					lua_pop(s, 1); // Pop the public table
					break;
				}
				default:
					log_err_at(
						node->src_loc, "Calling em.%s failed with error: %s", node->name->str, lua_tostring(s, -1));
					lua_pop(s, 1); // Pop the public table
					rc |= -1;
					break;
			}

			// Close variable scope
			lua_getglobal(s, CLOSE_VAR_SCOPE_FUNC_NAME);
			if (!is_callable(s, -1))
			{
				log_err_at(node->src_loc,
					"Variable " CLOSE_VAR_SCOPE_FUNC_NAME " is not a callable. Something has changed this!");
				lua_pop(s, 1);
				return -1;
			}
			if (lua_pcall(s, 0, 0, 0) != LUA_OK)
			{
				log_err_at(node->src_loc, "Failed to open new variable scope");
				return -1;
			}
			return rc;
		}
		case CONTENT:
		{
			ListIter li;
			make_list_iter(&li, node->content->content);
			DocTreeNode* subNode;
			while (iter_list((void**)&subNode, &li))
			{
				rc |= evaluate_directives(s, subNode, curr_iter);
				if (rc)
					break;
			}
			dest_list_iter(&li);
			return rc;
		}
		default:
			log_err_at(
				node->src_loc, "Failed to perform lua pass, encountered node of unknown type %d", node->content->type);
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

static int resolve_styling(DocTreeNode* node, Styler* sty)
{
	// Compute the style
	int rc = compute_style(sty, node);
	if (rc)
		return rc;

	switch (node->content->type)
	{
		case WORD:
			break;
		case CALL:
			if (node->content->call->result)
				return resolve_styling(node->content->call->result, sty);
			break;
		case CONTENT:
		{
			ListIter li;
			make_list_iter(&li, node->content->content);
			DocTreeNode* child;
			while (iter_list((void**)&child, &li))
				if ((rc = resolve_styling(child, sty)))
					return rc;
			break;
		}
		default:
			log_err_at(node->src_loc, "Failed to perform styling pass, encountered node of unknown type %d",
				node->content->type);
			return -1;
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
