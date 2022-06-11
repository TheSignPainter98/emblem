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

static int ext_eval_tree(ExtensionState* s)
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

	int erc = exec_ext_pass_on_node(s, sty, node, env->iter_num, false);
	if (erc)
		luaL_error(s, "Error while evaluating node");
	return 1;
}

int unpack_ext_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode)
{
	log_debug("Unpacking lua result");
	int rc;
	switch (lua_type(s, -1))
	{
		case LUA_TTABLE:
		{
			lua_getfield(s, -1, "_n");
			if (!lua_isuserdata(s, -1))
			{
				if (log_warn_at(parentNode->src_loc, "Directive result '_n' field did not contain a pointer"))
					return 1;
				*result = NULL;
			}

			rc = to_userdata_pointer((void**)result, s, -1, DOC_TREE_NODE);
			lua_pop(s, 2);
			return rc;
		}
		case LUA_TNIL:
			*result = NULL;
			lua_pop(s, 1);
			return 0;
		case LUA_TSTRING:
		{
			Str* repr = malloc(sizeof(Str));
			make_strc(repr, (char*)lua_tostring(s, -1));
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, 1);
			log_debug("Popped string '%s'", repr->str);
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
		case LUA_TBOOLEAN:
		{
			Str* repr = malloc(sizeof(Str));
			make_strv(repr, lua_toboolean(s, -1) ? "true" : "false");
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, 1);
			return rc;
		}
		case LUA_TUSERDATA:
		{
			if (!(rc = to_userdata_pointer((void**)result, s, -1, DOC_TREE_NODE)))
				connect_to_parent(*result, parentNode);
			lua_pop(s, 1);
			return rc;
		}
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
	make_doc_tree_node_word(*result, repr, dup_loc(parentNode->src_loc, false));
	(*result)->flags |= IS_GENERATED_NODE;
	connect_to_parent(*result, parentNode);
	return 0;
}

// TODO: complete re-implementation!

DocTreeNode* to_node(ExtensionState* s, int idx)
{
	DocTreeNode* ret = NULL;
	if (to_userdata_pointer((void**)&ret, s, idx, DOC_TREE_NODE))
		return luaL_error(s, "Expected doc-tree node"), NULL;
	return ret;
}

#define GET_LOCATION_IDX(s, idx, name)                                                                                 \
	lua_getfield(s, idx, #name);                                                                                       \
	size_t name = lua_tointeger(s, -1);                                                                                \
	lua_pop(s, 1);

static inline Location* to_location(ExtensionState* s, int idx)
{
	if (lua_isuserdata(s, idx))
	{
		Location* loc;
		if (!to_userdata_pointer((void**)&loc, s, idx, LOCATION))
			return dup_loc(loc, false);
		else
			return luaL_error(s, "Expected location"), NULL;
	}

	Location* ret;
	if (lua_isnil(s, idx))
	{
		ret		 = malloc(sizeof(Location));
		Str* src = malloc(sizeof(Str));
		make_strv(src, "(extension-space)");
		make_location(ret, 1, 1, 1, 1, src, true);
	}
	else if (lua_istable(s, idx))
	{
		ret = malloc(sizeof(Location));
		GET_LOCATION_IDX(s, idx, first_line);
		GET_LOCATION_IDX(s, idx, first_column);
		GET_LOCATION_IDX(s, idx, last_line);
		GET_LOCATION_IDX(s, idx, last_column);
		Str* src_file = malloc(sizeof(Str));
		lua_getfield(s, idx, "src_file");
		make_strc(src_file, lua_tostring(s, -1));
		lua_pop(s, 1);
		make_location(ret, first_line, first_column, last_line, last_column, src_file, true);
	}
	else
		return luaL_error(s, "Expected nil, userdata or table, got a %s", luaL_typename(s, idx)), NULL;
	return ret;
}

static int ext_get_node_id(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	lua_pushinteger(s, NODE_ID(node));
	return 1;
}

static int ext_get_node_flags(ExtensionState* s)
{
	DocTreeNodeFlags flags = to_node(s, 1)->flags;
	lua_pushinteger(s, flags & ACCEPTABLE_EXTENSION_FLAG_MASK);
	return 1;
}

static int ext_set_node_flags(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
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
	Str* name = to_node(s, 1)->name;
	lua_pushlstring(s, name->str, name->len);
	return 1;
}

static int ext_get_node_last_eval(ExtensionState* s)
{
	int last_eval = to_node(s, 1)->last_eval;
	lua_pushinteger(s, last_eval);
	return 1;
}

static int ext_get_node_parent(ExtensionState* s)
{
	DocTreeNode* parent = to_node(s, 1)->parent;
	push_doc_tree_node(s, parent);
	return 1;
}

static int ext_get_node_raw_word(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	if (node->content->type != WORD)
		return luaL_error(s, "Cannot extract raw word from node of type %d", node->content->type);
	Str* word = node->content->word->raw;
	lua_pushlstring(s, word->str, word->len);
	return 1;
}

static int ext_get_node_sanitised_word(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	if (node->content->type != WORD)
		return luaL_error(s, "Cannot extract raw word from node of type %d", node->content->type);
	Str* word = node->content->word->sanitised;
	lua_pushlstring(s, word->str, word->len);
	return 1;
}

static int ext_new_content_node(ExtensionState* s)
{
	Location* loc = to_location(s, 1);

	DocTreeNode* node;
	make_doc_tree_node_content(node = malloc(sizeof(DocTreeNode)), loc);
	push_doc_tree_node(s, node);
	return 1;
}

static int ext_new_word_node(ExtensionState* s)
{
	luaL_argcheck(s, true, lua_isstring(s, 1), "New word nodes need a string to represent");
	Location* loc = to_location(s, 2);

	Str* word;
	const char* raw = lua_tostring(s, 1);
	make_strc(word = malloc(sizeof(Str)), raw);
	DocTreeNode* node;
	make_doc_tree_node_word(node = malloc(sizeof(DocTreeNode)), word, loc);
	push_doc_tree_node(s, node);
	return 1;
}

static int ext_new_call_node(ExtensionState* s)
{
	luaL_argcheck(s, lua_isstring(s, 1), 1, "New call-nodes need a string call-name");
	luaL_argcheck(s, lua_istable(s, 2), 2, "New call-nodes need a list of arguments");
	Location* loc = to_location(s, 3);

	DocTreeNode* node = malloc(sizeof(DocTreeNode));
	Str* name		  = malloc(sizeof(Str));
	make_strc(name, lua_tostring(s, 1));
	CallIO* call = malloc(sizeof(CallIO));
	make_call_io(call);
	make_doc_tree_node_call(node, name, call, loc);

	lua_len(s, 2);
	int n_args = lua_tointeger(s, -1);
	lua_pop(s, 1);
	for (int i = 1; i <= n_args; i++)
	{
		lua_geti(s, 2, i);
		lua_getfield(s, -1, "_n");
		append_call_io_arg(call, to_node(s, -1));
		lua_pop(s, 2);
	}

	push_doc_tree_node(s, node);
	return 1;
}

static int ext_get_node_content_type(ExtensionState* s)
{
	lua_pushinteger(s, to_node(s, 1)->content->type);
	return 1;
}

static int ext_get_node_num_children(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	if (node->content->type != CONTENT)
		return luaL_error(s, "Cannot get content from %s node", node_tree_content_type_names[node->content->type]);
	lua_pushinteger(s, node->content->content->cnt);
	return 1;
}

static int ext_get_node_num_args(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	if (node->content->type != CALL)
		return luaL_error(s, "Cannot get args from %s node", node_tree_content_type_names[node->content->type]);
	lua_pushinteger(s, node->content->call->args->cnt);
	return 1;
}

static int ext_get_node_result(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	if (node->content->type != CALL)
		return luaL_error(s, "Cannot get result from %s node", node_tree_content_type_names[node->content->type]);
	DocTreeNode* result = node->content->call->result;
	if (result)
		push_doc_tree_node(s, result);
	else
		lua_pushnil(s);
	return 1;
}

static int ext_get_node_child(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	luaL_argcheck(s, true, lua_isnumber(s, 2), "Index of child must be a number");
	if (node->content->type != CONTENT)
		return luaL_error(s, "Cannot get children of %s node", node_tree_content_type_names[node->content->type]);

	Maybe m;
	get_list_elem(&m, node->content->content, lua_tointeger(s, 2) - 1);
	switch (m.type)
	{
		case NOTHING:
			lua_pushnil(s);
			break;
		case JUST:
			push_doc_tree_node(s, (DocTreeNode*)m.just);
			break;
	}
	return 1;
}

static int ext_get_node_arg(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	luaL_argcheck(s, true, lua_isnumber(s, 2), "Index of argument must be a number");
	if (node->content->type != CALL)
		return luaL_error(s, "Cannot get arguments of %s node", node_tree_content_type_names[node->content->type]);

	Maybe m;
	get_list_elem(&m, node->content->call->args, lua_tointeger(s, 2) - 1);
	switch (m.type)
	{
		case NOTHING:
			lua_pushnil(s);
			break;
		case JUST:
			push_doc_tree_node(s, (DocTreeNode*)m.just);
			break;
	}
	return 1;
}

static int ext_get_node_style(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	pack_style(s, node->style, node);
	return 1;
}

static int ext_get_node_attr(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	luaL_argcheck(s, lua_isstring(s, 2), 2, "Attribute-getting requires string key");
	if (node->content->type != CALL)
		return luaL_error(s, "Cannot get attributes from %s node", node_tree_content_type_names[node->content->type]);

	Maybe m;
	Str k;
	make_strv(&k, lua_tostring(s, 2));
	get_attr(&m, node->content->call->attrs, &k);
	switch (m.type)
	{
		case NOTHING:
			lua_pushnil(s);
			break;
		case JUST:
			lua_pushstring(s, ((Str*)m.just)->str);
			break;
	}
	dest_str(&k);
	return 1;
}

static int ext_set_node_attr(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	luaL_argcheck(s, lua_isstring(s, 2), 2, "Attribute setting requires string key");
	luaL_argcheck(s, lua_isstring(s, 3), 3, "Attibute values must be strings");
	if (node->content->type != CALL)
		return luaL_error(s, "Cannot set attributes of %s node", node_tree_content_type_names[node->content->type]);

	Str* k;
	Str* v;
	make_strc(k = malloc(sizeof(Str)), lua_tostring(s, 2));
	make_strc(v = malloc(sizeof(Str)), lua_tostring(s, 3));

	set_attr(&node->content->call->attrs, k, v);
	return 0;
}

static int ext_append_node_child(ExtensionState* s)
{
	DocTreeNode* node			= to_node(s, 1);
	DocTreeNodeContentType type = node->content->type;
	if (type != CONTENT)
		return luaL_error(s, "Can only append to node of type %s: got a %s", node_tree_content_type_names[CONTENT],
			node_tree_content_type_names[type]);
	DocTreeNode* new_child = to_node(s, 2);
	connect_to_parent(new_child, node);
	return 0;
}

static int ext_append_call_arg(ExtensionState* s)
{
	DocTreeNode* node			= to_node(s, 1);
	DocTreeNodeContentType type = node->content->type;
	if (type != CALL)
		return luaL_error(s, "Can only append argument to %s node: got a %s", node_tree_content_type_names[CALL],
			node_tree_content_type_names[type]);
	DocTreeNode* new_arg = to_node(s, 2);
	append_call_io_arg(node->content->call, new_arg);
	return 0;
}

static int ext_get_node_location(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	push_location(s, node->src_loc);
	return 1;
}

static int ext_copy_node(ExtensionState* s)
{
	DocTreeNode* node = to_node(s, 1);
	push_doc_tree_node(s, copy_doc_tree_node(node));
	return 1;
}

void register_ext_node(ExtensionState* s)
{
	register_api_table(s, "__node", {
		register_api_function(s, "__eval", ext_eval_tree);
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
		register_api_function(s, "__get_num_args", ext_get_node_num_args);
		register_api_function(s, "__get_num_children", ext_get_node_num_children);
		register_api_function(s, "__get_result", ext_get_node_result);
		register_api_function(s, "__get_child", ext_get_node_child);
		register_api_function(s, "__get_arg", ext_get_node_arg);
		register_api_function(s, "__get_style", ext_get_node_style);
		register_api_function(s, "__get_attr", ext_get_node_attr);
		register_api_function(s, "__set_attr", ext_set_node_attr);
		register_api_function(s, "__append_child", ext_append_node_child);
		register_api_function(s, "__append_arg", ext_append_call_arg);
		register_api_function(s, "__get_loc", ext_get_node_location);
		register_api_function(s, "__copy", ext_copy_node);
		register_api_function(s, "__get_id", ext_get_node_id);
	});
}
