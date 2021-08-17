#include "lua-pointers.h"

#include "logs/logs.h"
#include <lua.h>
#include <lauxlib.h>

const char* const lua_pointer_type_names[] = {
	[AST_NODE] = "AST node",
	[STYLER] = "styler",
	[EXT_ENV] = "extension environment",
	[MT_NAMES_LIST] = "mt-safe file-name list",
	[PARSED_ARGS] = "parsed command-line arguments",
};

void make_lua_pointer(LuaPointer* pointer, LuaPointerType type, void* data)
{
	pointer->type = type;
	pointer->data = data;
}

void dest_lua_pointer(LuaPointer* pointer, Destructor ed)
{
	if (ed)
		ed(pointer->data);
}
