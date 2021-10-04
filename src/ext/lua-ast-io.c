/**
 * @file lua-ast-io.c
 * @brief Provides functions for translating from Lua tables to docuemnt-trees and vice-versa
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "lua-ast-io.h"

#include <lauxlib.h>
#include <luaconf.h>

#include "data/str.h"
#include "doc-struct/ast.h"
#include "logs/logs.h"
#include "lua-pointers.h"
#include "lua.h"
#include "ext-env.h"

#include "debug.h"

static int unpack_single_value(DocTreeNode** result, Str* repr, DocTreeNode* parentNode);
static int unpack_table_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode);

int ext_eval_tree(ExtensionState* s)
{
	if (lua_gettop(s) != 1)
		return luaL_error(s, "Expected exactly 1 argument");

	DocTreeNode* node;
	int rc = to_userdata_pointer((void**)&node, s, -1, AST_NODE);
	if (rc)
		luaL_error(s, "Invalid argument(s)");
	lua_pop(s, 1);
	log_debug("Working on node %p", (void*)node);

	lua_getglobal(s, EM_ENV_VAR_NAME);
	ExtensionEnv* env;
	rc = to_userdata_pointer((void**)&env, s, -1, EXT_ENV);
	if (rc)
		luaL_error(s, "Invalid argument(s)");
	lua_pop(s, 1);

	int erc = exec_lua_pass_on_node(s, node, env->iter_num);
	if (erc)
		luaL_error(s, "Error while evaluating node");
	else
		pack_tree(s, node);
	return 1;
}

int pack_tree(ExtensionState* s, DocTreeNode* node)
{
	int rc = 0;

	log_debug("[");
	log_debug("Evaluating tree at %p...", (void*)node);
	dumpstack(s);
	lua_newtable(s);
	lua_pushinteger(s, node->content->type);
	lua_setfield(s, -2, "type");

	lua_pushinteger(s, node->flags & ACCEPTABLE_EXTENSION_FLAG_MASK);
	lua_setfield(s, -2, "flags");

	switch (node->content->type)
	{
		case WORD:
			lua_pushlstring(s, node->content->word->str, node->content->word->len);
			lua_setfield(s, -2, "word");
			break;
		case CALL:
			lua_pushlstring(s, node->name->str, node->name->len);
			lua_setfield(s, -2, "name");

			// Pack the arguments
			ListIter li;
			make_list_iter(&li, node->content->call->args);
			DocTreeNode* curr_arg;
			int arg_idx = 1;
			lua_createtable(s, node->content->call->args->cnt, 0); // NOLINT
			while (iter_list((void**)&curr_arg, &li))
			{
				rc = pack_tree(s, curr_arg);
				if (rc)
					return rc;
				lua_seti(s, -2, arg_idx++);
			}
			lua_setfield(s, -2, "args");

			// Pack the result if present
			if (node->content->call->result)
			{
				log_debug("Packing result of %p found at %p", (void*)node, (void*)node->content->call->result);
				rc = pack_tree(s, node->content->call->result);
				log_debug("Done packing %p!", (void*)node->content->call->result);
			}
			else
				lua_pushnil(s);
			lua_setfield(s, -2, "result");
			break;
		case CONTENT:
		{
			DocTreeNode* child;
			ListIter li;
			make_list_iter(&li, node->content->content);
			int idx = 1;
			lua_newtable(s);
			while (iter_list((void**)&child, &li))
			{
				pack_tree(s, child);
				lua_seti(s, -2, idx++);
			}
			lua_setfield(s, -2, "content");
			break;
		}
	}
	log_debug("]");
	return rc;
}

int unpack_lua_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode)
{
	log_debug("Unpacking lua result, stack is:");
	dumpstack(s);
	int rc;
	switch (lua_type(s, -1))
	{
		case LUA_TNIL:
			*result = NULL;
			lua_pop(s, 1);
			return 0;
		case LUA_TBOOLEAN:
		{
			Str* repr = malloc(sizeof(Str));
			make_strv(repr, lua_toboolean(s, -1) ? "true" : "false");
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, 1);
			return rc;
		}
		case LUA_TNUMBER:
		{
			char* numr;
			if (lua_isinteger(s, -1))
			{
				lua_Integer num		= lua_tonumber(s, -1);
				const size_t numlen = 1 + snprintf(NULL, 0, LUA_INTEGER_FMT, num);
				numr				= malloc(numlen);
				snprintf(numr, numlen, LUA_INTEGER_FMT, num);
			}
			else
			{
				lua_Number num		= lua_tonumber(s, -1);
				const size_t numlen = 1 + snprintf(NULL, 0, LUA_NUMBER_FMT, num);
				numr				= malloc(numlen);
				snprintf(numr, numlen, LUA_NUMBER_FMT, num);
			}
			Str* repr = malloc(sizeof(Str));
			make_strr(repr, numr);
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, 1);
			return rc;
		}
		case LUA_TSTRING:
		{
			Str* repr = malloc(sizeof(Str));
			make_strv(repr, (char*)lua_tostring(s, -1));
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, 1);
			log_debug("Popped string '%s'", repr->str);
			return rc;
		}
		case LUA_TLIGHTUSERDATA:
		{
			LuaPointer* p = lua_touserdata(s, -1);
			if (p->type != AST_NODE)
			{
				log_err("Could not unpack light user data %s, had type %d but expected %d", lua_tostring(s, -1),
					p->type, AST_NODE);
				return -1;
			}
			log_debug("Passing reference to %p", p->data);
			*result			  = p->data;
			(*result)->parent = parentNode;
			lua_pop(s, 1);
			return 0;
		}
		case LUA_TTABLE:
			return unpack_table_result(result, s, parentNode);
		default:
		{
			const char* repr = lua_tostring(s, -1);
			log_err("Cannot read a %s (got %s)", luaL_typename(s, -1), repr);
			*result = NULL;
			lua_pop(s, 1);
			return -1;
		}
	}
}

static int unpack_single_value(DocTreeNode** result, Str* repr, DocTreeNode* parentNode)
{
	*result = malloc(sizeof(DocTreeNode));
	make_doc_tree_node_word(*result, repr, dup_loc(parentNode->src_loc));
	(*result)->flags |= IS_GENERATED_NODE;
	(*result)->parent = parentNode;
	return 0;
}

// Takes the value at the value at the top of the stack, unpacks and pops it [-1, +d, m], for tree of depth d
static int unpack_table_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode)
{
	int rc;
	log_info("Unpacking table...");

	// Get the flags
	lua_getfield(s, -1, "flags");
	int in_flags = lua_tointeger(s, -1);
	{
		int bad_flags = in_flags & ~ACCEPTABLE_EXTENSION_FLAG_MASK;
		if (bad_flags)
			if (log_warn_at(parentNode->src_loc, "Ignoring invalid flags when unpacking table-representation of a node: %x", bad_flags))
				return 1;
	}
	int flags = IS_GENERATED_NODE | (ACCEPTABLE_EXTENSION_FLAG_MASK & lua_tointeger(s, -1));
	lua_pop(s, 1);

	lua_getfield(s, -1, "type");
	if (!lua_isinteger(s, -1))
	{
		log_err_at(parentNode->src_loc, "The contents of a 'type' field must be an integer, got a %s instead (%s)",
			luaL_typename(s, -1), lua_tostring(s, -1));
		lua_pop(s, 2);
		return -1;
	}
	DocTreeNodeContentType type = lua_tointeger(s, -1);
	lua_pop(s, 1);
	switch (type)
	{
		case WORD:
			lua_getfield(s, -1, "word");
			char* word	 = (char*)lua_tostring(s, -1);
			Str* wordstr = malloc(sizeof(Str));
			make_strv(wordstr, word);
			rc = unpack_single_value(result, wordstr, parentNode);
			(*result)->flags = flags;
			lua_pop(s, 1);
			break;
		case CONTENT:
			*result = malloc(sizeof(DocTreeNode));
			make_doc_tree_node_content(*result, dup_loc(parentNode->src_loc));
			(*result)->flags = flags;
			// Iterate over the 'content' field list, unpacking at each level
			lua_getfield(s, -1, "content");
			lua_pushnil(s); /* first key */
			while (lua_next(s, -2) != 0)
			{
				/* Uses key at index -2 and value at index -1 */
				log_info("Iterating on content part %s", lua_tostring(s, -1));
				DocTreeNode* new_child;
				int rc = unpack_lua_result(&new_child, s, *result);
				if (rc)
					return rc; // NOLINT
				append_doc_tree_node_child(*result, (*result)->content->content, new_child);
			}
			lua_pop(s, 1);
			rc = 0;
			break;
		case CALL:
			*result = malloc(sizeof(DocTreeNode));
			CallIO* io = malloc(sizeof(CallIO));
			make_call_io(io);

			// Extract call name
			Str* call_name = malloc(sizeof(Str));
			lua_getfield(s, -1, "name");
			make_strv(call_name, (char*)lua_tostring(s, -1));
			lua_pop(s, 1);

			// Extract arguments
			lua_getfield(s, -1, "args");
			if (lua_type(s, -1) != LUA_TTABLE)
			{
				log_err("Attempted to unpack 'args' field of a call, but got %s object (%s)", luaL_typename(s, -1), lua_tostring(s, -1));
				lua_pop(s, 1);
				rc = 1;
				break;
			}
			make_doc_tree_node_call(*result, call_name, io, dup_loc(parentNode->src_loc));
			(*result)->flags = flags;
			dumpstack(s);
			lua_len(s, -1);
			int num_args = lua_tointeger(s, -1);
			lua_pop(s, 1);
			for (int i = 1; i <= num_args; i++)
			{
				lua_geti(s, -1, i);
				DocTreeNode* arg;
				int rc = unpack_lua_result(&arg, s, *result);
				if (rc)
					return rc; // NOLINT
				append_call_io_arg(io, arg);
			}

			// Unpack the result
			lua_getfield(s, -1, "result");
			if (lua_type(s, -1) != LUA_TNIL)
				unpack_lua_result(&(*result)->content->call->result, s, *result);
			else
				(*result)->content->call->result = NULL;
			lua_pop(s, 2);
			rc = 0;
			break;
		default:
			log_err_at(parentNode->src_loc, "Unknown node type %d, failed to unpack table", type);
			return -1;
	}
	lua_pop(s, 1);
	return rc;
}
