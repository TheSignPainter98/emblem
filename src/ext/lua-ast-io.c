#include "lua-ast-io.h"

#include <lauxlib.h>

#include "doc-struct/ast.h"
#include "logs/logs.h"
#include "lua-pointers.h"
#include "lua.h"

#include "debug.h"

static int eval_tree(ExtensionState* s, DocTreeNode* node);
static int unpack_single_value(DocTreeNode** result, Str* repr, DocTreeNode* parentNode);

int ext_eval_tree(ExtensionState* s)
{
	if (lua_gettop(s) != 1)
		return luaL_error(s, "Expected exactly 1 argument");

	if (!lua_isuserdata(s, -1))
		luaL_error(s, "Expected userdata but got %s '%s'", luaL_typename(s, -1), lua_tostring(s, -1));
	LuaPointer* ptr = lua_touserdata(s, -1);
	if (ptr->type != AST_NODE)
		luaL_error(s, "Expected AST node userdata (%d) but got userdata of type %d", AST_NODE, ptr->type);
	DocTreeNode* node = ptr->data;
	lua_pop(s, 1);
	log_debug("Working on node %p", (void*)node);

	eval_tree(s, node);
	return 1;
}

#define EVAL_TREE_CASE(L, NAME, name)                                                                                  \
	case NAME:                                                                                                         \
	{                                                                                                                  \
		DocTreeNode* child;                                                                                            \
		ListIter li;                                                                                                   \
		make_list_iter(&li, node->content->name);                                                                      \
		int idx = 1;                                                                                                   \
		lua_newtable(L);                                                                                               \
		while (iter_list((void**)&child, &li))                                                                         \
		{                                                                                                              \
			eval_tree(L, child);                                                                                       \
			lua_seti(s, -2, idx++);                                                                                    \
		}                                                                                                              \
		lua_setfield(L, -2, "" #name);                                                                                 \
		break;                                                                                                         \
	}

static int eval_tree(ExtensionState* s, DocTreeNode* node)
{
	lua_newtable(s);
	lua_pushinteger(s, node->content->type);
	lua_setfield(s, -2, "type");
	switch (node->content->type)
	{
		case WORD:
			lua_pushstring(s, node->content->word->str);
			lua_setfield(s, -2, "word");
			break;
		case CALL:
			lua_pushstring(s, node->name->str);
			lua_setfield(s, -2, "name");
			/* TODO: does this work? */
			if (log_warn("Packing node %p by executing its lua pass", (void*)node))
				luaL_error(s, "Warnings are fatal");
			int rc = exec_lua_pass_on_node(s, node);
			if (!rc && node->content->call_params->result)
			{
				log_debug("Packing %p!", (void*)node->content->call_params->result);
				rc = eval_tree(s, node->content->call_params->result);
				dumpstack(s);
				log_debug("Done packing %p!", (void*)node->content->call_params->result);
			}
			else
				lua_pushnil(s);
			dumpstack(s);
			log_debug("Setting result...");
			lua_setfield(s, -2, "result");
			log_debug("Set result!");
			return rc;
			break;
			EVAL_TREE_CASE(s, LINE, line);
			EVAL_TREE_CASE(s, LINES, lines);
			EVAL_TREE_CASE(s, PAR, par);
			EVAL_TREE_CASE(s, PARS, pars);
	}
	return 0;
}

int get_ast_type_name(ExtensionState* s)
{
	luaL_argcheck(s, true, lua_gettop(s) == 1, "Expected exactly one argument to ast_type_name");
	luaL_argcheck(s, true, lua_isinteger(s, -1), "Expected integer argument to ast_type_name");

	int tn = lua_tointeger(s, -1);
	log_debug("Looking at ast type id %d", tn);
	luaL_argcheck(s, true, WORD <= tn && tn <= PARS, "Type index is not in the valid range");
	const char* type_names[] = {
		[WORD]	= "word",
		[CALL]	= "call",
		[LINE]	= "line",
		[LINES] = "lines",
		[PAR]	= "par",
		[PARS]	= "pars",
	};
	lua_pushstring(s, type_names[tn]);
	return 1;
}

int unpack_lua_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode)
{
	luaL_argcheck(s, true, lua_gettop(s) == 1, "Expected exactly one result");
	int rc;
	switch (lua_type(s, -1))
	{
		case LUA_TNIL:
			*result = NULL;
			return 0;
		case LUA_TBOOLEAN:
		{
			Str* repr = malloc(sizeof(Str));
			make_strv(repr, lua_toboolean(s, -1) ? "true" : "false");
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, -1);
			return rc;
		}
		case LUA_TNUMBER:
		{
			const int num		= lua_tonumber(s, -1);
			const size_t numlen = 1 + snprintf(NULL, 0, "%d", num);
			char* numr			= malloc(numlen);
			snprintf(numr, numlen, "%d", num);
			Str* repr = malloc(sizeof(Str));
			make_strr(repr, numr);
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, -1);
			return rc;
		}
		case LUA_TSTRING:
		{
			Str* repr = malloc(sizeof(Str));
			make_strv(repr, (char*)lua_tostring(s, -1));
			rc = unpack_single_value(result, repr, parentNode);
			lua_pop(s, -1);
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
			lua_pop(s, -1);
			return 0;
		}
		case LUA_TTABLE:
			/* return unpack_table_result(result, s, parent_node); */
			log_err("Tables are not currently supported.");
			lua_pop(s, -1);
			return -1;
		default:
		{
			const char* repr = lua_tostring(s, -1);
			log_err("Cannot read a %s (got %s)", luaL_typename(s, -1), repr);
			*result = NULL;
			lua_pop(s, -1);
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
