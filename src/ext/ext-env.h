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
#define EM_ENV_VAR_NAME			  "__env"
#define EM_ARGS_VAR_NAME		  "__args"
#define EM_MT_NAMES_LIST_VAR_NAME "__mt_names_list"
#define EM_API_TABLE_NAME		  "__em"

#define register_api_function(s, name, f)                                                                              \
	{                                                                                                                  \
		lua_pushcfunction(s, f);                                                                                       \
		lua_setfield(s, -2, name);                                                                                     \
	}

#define register_api_table(s, name, api)                                                                               \
	{                                                                                                                  \
		lua_newtable(s);                                                                                               \
		api;                                                                                                           \
		lua_setfield(s, -2, name);                                                                                     \
	}

extern const char* const lua_pointer_type_names[];

typedef enum
{
	DOC_TREE_NODE,
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
LuaPointer* new_lua_pointer(ExtensionState* s, LuaPointerType type, void* data);
void release_pass_local_lua_pointers(ExtensionEnv* e);
int to_userdata_pointer(void** val, ExtensionState* s, int idx, LuaPointerType type);

void get_api_elem(ExtensionState* s, const char* name);
void set_api_elem(ExtensionState* s, int idx, const char* name);
void update_api_elem(ExtensionState* s, int idx, const char* name);
