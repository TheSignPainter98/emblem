/**
 * @file ext-env.h
 * @brief Exposes functions for handling the Lua extension environment
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-params.h"
#include <lualib.h>

#define EM_PUBLIC_TABLE			  "em"
#define EM_ITER_NUM_VAR_NAME	  "em_iter"
#define EM_ENV_VAR_NAME			  "_em_env"
#define EM_ARGS_VAR_NAME		  "_em_args"
#define EM_MT_NAMES_LIST_VAR_NAME "_em_mt_names_list"

extern const char* const lua_pointer_type_names[];

typedef enum
{
	AST_NODE,
	STYLER,
	EXT_ENV,
	MT_NAMES_LIST,
	PARSED_ARGS,
	LOCATION,
} LuaPointerType;

typedef struct
{
	LuaPointerType type;
	void* data;
	bool valid;
	bool destruction_permitted;
} LuaPointer;

typedef lua_State ExtensionState;
typedef struct
{
	ExtensionState* state;
	int iter_num;
	bool require_extra_run;
} ExtensionEnv;

int make_ext_env(ExtensionEnv* ext, ExtParams* params);
void dest_ext_env(ExtensionEnv* ext);
LuaPointer* new_lua_pointer(ExtensionState* s, LuaPointerType type, void* data, bool destruction_permitted);
void release_pass_local_lua_pointers(ExtensionEnv* e);
void invalidate_lua_pointer(LuaPointer* lp);
int to_userdata_pointer(void** val, ExtensionState* s, int idx, LuaPointerType type);
