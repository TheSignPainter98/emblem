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
#include "ext-env.h"
#include "logs/logs.h"
#include "lua.h"
#include "style.h"

#include "debug.h"

static int unpack_single_value(DocTreeNode** result, Str* repr, DocTreeNode* parentNode);
static int unpack_table_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode);

int ext_eval_tree(ExtensionState* s)
{
	if (lua_gettop(s) != 1)
		return luaL_error(s, "Expected exactly 1 argument");

	DocTreeNode* node;
	int rc = to_userdata_pointer((void**)&node, s, -1, DOC_TREE_NODE);
	if (rc)
		luaL_error(s, "Invalid argument(s)");
	lua_pop(s, 1);
	log_debug("Working on node %p", (void*)node);

	get_api_elem(s, EM_ENV_VAR_NAME);
	ExtensionEnv* env;
	rc = to_userdata_pointer((void**)&env, s, -1, EXT_ENV);
	if (rc)
		luaL_error(s, "Invalid argument(s)");
	lua_pop(s, 1);

	get_api_elem(s, EM_STYLER_LP_LOC);
	Styler* sty;
	rc = to_userdata_pointer((void**)&sty, s, -1, STYLER);
	if (rc)
		luaL_error(s, "Invalid styler value");
	lua_pop(s, 1);

	int erc = exec_lua_pass_on_node(s, sty, node, env->iter_num, false);
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

	pack_style(s, node->style, node);
	lua_setfield(s, -2, "style");

	switch (node->content->type)
	{
		case WORD:
			Word* word = node->content->word;
			lua_pushlstring(s, word->raw->str, word->raw->len);
			lua_setfield(s, -2, "word");
			lua_pushlstring(s, word->sanitised->str, word->sanitised->len);
			lua_setfield(s, -2, "pword");
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

			if (node->content->call->attrs)
			{
				Attrs* attrs = node->content->call->attrs;
				lua_createtable(s, 0, attrs->curr_stored);
				MapIter mi;
				make_map_iter(&mi, attrs);
				Pair* kv;
				while (iter_map(&kv, &mi))
				{
					Str* k = kv->p0;
					Str* v = kv->p1;
					lua_pushlstring(s, v->str, v->len);
					lua_setfield(s, -2, k->str);
				}
				dest_map_iter(&mi);
				lua_setfield(s, -2, "attrs");
			}
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
			make_strc(repr, (char*)lua_tostring(s, -1));
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, 1);
			log_debug("Popped string '%s'", repr->str);
			return rc;
		}
		case LUA_TUSERDATA:
		{
			if (!to_userdata_pointer((void**)result, s, -1, DOC_TREE_NODE))
				connect_to_parent(*result, parentNode);
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
	connect_to_parent(*result, parentNode);
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
			if (log_warn_at(parentNode->src_loc,
					"Ignoring invalid flags when unpacking table-representation of a node: %x", bad_flags))
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
			make_strc(wordstr, word);
			rc				 = unpack_single_value(result, wordstr, parentNode);
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
			}
			connect_to_parent(*result, parentNode);
			lua_pop(s, 1);
			rc = 0;
			break;
		case CALL:
			*result	   = malloc(sizeof(DocTreeNode));
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
				log_err("Attempted to unpack 'args' field of a call, but got %s object (%s)", luaL_typename(s, -1),
					lua_tostring(s, -1));
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
			connect_to_parent(*result, parentNode);
			lua_pop(s, 1);

			// Unpack attributes
			Attrs* attrs = io->attrs = malloc(sizeof(Attrs));
			make_attrs(attrs);
			lua_getfield(s, -1, "attrs");
			if (!lua_isnil(s, -1))
			{
				lua_pushnil(s);
				while (lua_next(s, -2))
				{
					Str* k = malloc(sizeof(Str));
					Str* v = malloc(sizeof(Str));
					make_strc(k, lua_tostring(s, -2));
					make_strc(v, lua_tostring(s, -1));
					set_attr(attrs, k, v);
					lua_pop(s, 1);
				}
			}
			lua_pop(s, 1);

			rc = 0;
			break;
		default:
			log_err_at(parentNode->src_loc, "Unknown node type %d, failed to unpack table", type);
			return -1;
	}
	lua_pop(s, 1);
	return rc;
}

// TODO: complete re-implementation!

static inline DocTreeNode* ensure_first_arg_is_node(ExtensionState* s)
{
	DocTreeNode* ret = NULL;
	luaL_argcheck(s, true, !to_userdata_pointer((void**)&ret, s, 1, DOC_TREE_NODE), "Expected node argument");
	return ret;
}

static int ext_get_node_flags(ExtensionState* s)
{
	DocTreeNodeFlags flags = ensure_first_arg_is_node(s)->flags;
	lua_pushinteger(s, flags & ACCEPTABLE_EXTENSION_FLAG_MASK);
	return 1;
}

static int ext_set_node_flags(ExtensionState* s)
{
	DocTreeNode* node = ensure_first_arg_is_node(s);
	luaL_argcheck(s, true, lua_isinteger(s, 2), "Node flags must be an integer value");

	DocTreeNodeFlags flags = lua_tointeger(s, 2);
	{
		DocTreeNodeFlags bad_flags = flags & ~ACCEPTABLE_EXTENSION_FLAG_MASK;
		if (bad_flags)
			if (log_warn("Ignoring invalid flags: %x", bad_flags))
				return 1;
	}

	node->flags = (node->flags & ~ACCEPTABLE_EXTENSION_FLAG_MASK) | (flags & ACCEPTABLE_EXTENSION_FLAG_MASK);

	return 0;
}

static int ext_get_node_name(ExtensionState* s)
{
	Str* name = ensure_first_arg_is_node(s)->name;
	lua_pushlstring(s, name->str, name->len);
	return 1;
}

static int ext_get_node_last_eval(ExtensionState* s)
{
	int last_eval = ensure_first_arg_is_node(s)->last_eval;
	lua_pushinteger(s, last_eval);
	return 1;
}

static int ext_get_node_parent(ExtensionState* s)
{
	DocTreeNode* parent = ensure_first_arg_is_node(s)->parent;
	get_doc_tree_node_lua_pointer(s, parent);
	return 1;
}

static int ext_get_node_raw_word(ExtensionState* s)
{
	DocTreeNode* node = ensure_first_arg_is_node(s);
	if (node->content->type != WORD)
		return luaL_error(s, "Cannot extract raw word from node of type %d", node->content->type);
	Str* word = node->content->word->raw;
	lua_pushlstring(s, word->str, word->len);
	return 1;
}

static int ext_get_node_sanitised_word(ExtensionState* s)
{
	DocTreeNode* node = ensure_first_arg_is_node(s);
	if (node->content->type != WORD)
		return luaL_error(s, "Cannot extract raw word from node of type %d", node->content->type);
	Str* word = node->content->word->sanitised;
	lua_pushlstring(s, word->str, word->len);
	return 1;
}

static int ext_new_content_node(ExtensionState* s)
{
	DocTreeNode* node;
	make_doc_tree_node_content(node = malloc(sizeof(DocTreeNode)), NULL); // TODO: get the location!
	get_doc_tree_node_lua_pointer(s, node);
	return 1;
}

static int ext_new_word_node(ExtensionState* s)
{
	luaL_argcheck(s, true, lua_isstring(s, 1), "New word nodes need a string to represent");

	Str* word;
	const char* raw = lua_tostring(s, 1);
	make_strc(word = malloc(sizeof(Str)), raw);
	DocTreeNode* node;
	make_doc_tree_node_word(node = malloc(sizeof(DocTreeNode)), word, NULL); // TODO: get the location!
	get_doc_tree_node_lua_pointer(s, node);
	return 1;
}

static int ext_new_call_node(ExtensionState* s)
{
	luaL_argcheck(s, true, lua_isstring(s, 1), "New call-nodes need a string call-name as the first argument");
	// TODO: complete the implementation of this!
	return 1;
}

static int ext_get_node_content_type(ExtensionState* s)
{
	lua_pushinteger(s, ensure_first_arg_is_node(s)->content->type);
	return 1;
}

static int ext_get_node_num_children(ExtensionState* s)
{
	DocTreeNode* node = ensure_first_arg_is_node(s);
	if (node->content->type != CONTENT)
		return luaL_error(s, "Cannot get content from %s node", node_tree_content_type_names[node->content->type]);
	lua_pushinteger(s, node->content->content->cnt);
	return 1;
}

void register_ext_node(ExtensionState* s)
{
	register_api_table(s, "__node", {
		register_api_function(s, "__get_flags", ext_get_node_flags);
		register_api_function(s, "__set_flags", ext_set_node_flags);
		register_api_function(s, "__get_name", ext_get_node_name);
		register_api_function(s, "__get_last_eval", ext_get_node_last_eval);
		register_api_function(s, "__get_sanitised_word", ext_get_node_sanitised_word);
		register_api_function(s, "__get_raw_word", ext_get_node_raw_word);
		register_api_function(s, "__get_parent", ext_get_node_parent);
		register_api_function(s, "__new_word", ext_new_word_node);
		register_api_function(s, "__new_content", ext_new_content_node);
		register_api_function(s, "__new_call", ext_new_call_node);
		register_api_function(s, "__get_content_type", ext_get_node_content_type);
		register_api_function(s, "__get_num_children", ext_get_node_num_children);
	});
}
