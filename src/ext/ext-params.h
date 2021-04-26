#pragma once

#include "argp.h"
#include "lua-pointers.h"
#include <lua.h>
#include <stdbool.h>

typedef lua_State ExtensionState;
typedef struct
{
	ExtensionState* state;
	LuaPointer* styler;
	LuaPointer* selfp;
	int iter_num;
	bool require_extra_run;
} ExtensionEnv;

typedef struct
{
	int sandbox_lvl;
} ExtParams;

void init_ext_params(ExtParams* params, Args* args);
void dest_ext_params(ExtParams* params);
