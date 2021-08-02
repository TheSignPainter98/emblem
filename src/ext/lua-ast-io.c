#include "lua-ast-io.h"

#include <lauxlib.h>
#include <luaconf.h>

#include "doc-struct/ast.h"
#include "logs/logs.h"
#include "lua-pointers.h"
#include "lua.h"

#include "debug.h"

static int pack_tree(ExtensionState* s, DocTreeNode* node);
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
	lua_pop(s, -1);
	log_debug("Working on node %p", (void*)node);

	int erc = exec_lua_pass_on_node(s, node);
	if (erc)
		lua_pushnil(s);
	pack_tree(s, node);
	return 1;
}

static int pack_tree(ExtensionState* s, DocTreeNode* node)
{
	int rc = 0;

	log_debug("[");
	log_debug("Evaluating tree at %p...", (void*)node);
	dumpstack(s);
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
			if (node->content->call->result)
			{
				log_debug("Packing result of %p found at %p", (void*)node, (void*)node->content->call->result);
				rc = pack_tree(s, node->content->call->result);
				log_debug("Post-pack stack");
				dumpstack(s);
				log_debug("Done packing %p!", (void*)node->content->call->result);
			}
			else
				lua_pushnil(s);
			dumpstack(s);
			log_debug("Setting result...");
			lua_setfield(s, -2, "result");
			log_debug("Set result!");
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
	luaL_argcheck(s, true, lua_gettop(s) == 1, "Expected exactly one result");
	int rc;
	switch (lua_type(s, -1))
	{
		case LUA_TNIL:
			*result = NULL;
			lua_pop(s, -1);
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
				numr				= malloc(sizeof(numlen));
				snprintf(numr, numlen, LUA_NUMBER_FMT, num);
			}
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
