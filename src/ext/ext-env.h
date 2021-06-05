#pragma once

#include <lualib.h>
#include "ext-params.h"
#include "lua-pointers.h"

#define EM_PUBLIC_TABLE			   "em"
#define EM_ITER_NUM_VAR_NAME	   "em_iter"

typedef lua_State ExtensionState;
typedef struct
{
	ExtensionState* state;
	LuaPointer* styler;
	LuaPointer* selfp;
	int iter_num;
	bool require_extra_run;
} ExtensionEnv;

int make_ext_env(ExtensionEnv* ext, ExtParams* params);
void dest_ext_env(ExtensionEnv* ext);
void finalise_env_for_typesetting(ExtensionEnv* e);
