/**
 * @file ext-env.h
 * @brief Exposes functions for handling the Lua extension environment
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-params.h"
#include "lua-pointers.h"
#include <lualib.h>

#define EM_PUBLIC_TABLE			  "em"
#define EM_ITER_NUM_VAR_NAME	  "em_iter"
#define EM_ENV_VAR_NAME			  "_em_env"
#define EM_ARGS_VAR_NAME		  "_em_args"
#define EM_MT_NAMES_LIST_VAR_NAME "_em_mt_names_list"

typedef lua_State ExtensionState;
typedef struct
{
	ExtensionState* state;
	LuaPointer* styler;
	LuaPointer* selfp;
	LuaPointer* args;
	LuaPointer* mt_names_list;
	int iter_num;
	bool require_extra_run;
} ExtensionEnv;

int make_ext_env(ExtensionEnv* ext, ExtParams* params);
void dest_ext_env(ExtensionEnv* ext);
void finalise_env_for_typesetting(ExtensionEnv* e);
