#include "lua-pointers.h"

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
