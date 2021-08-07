#pragma once

#include "argp.h"
#include "data/locked.h"
#include "style/css-params.h"
#include <lua.h>
#include <stdbool.h>

typedef struct
{
	int sandbox_lvl;
	List* exts;
	List* ext_args;
	Styler* styler;
	Args* args;
	Locked* mt_names_list;
} ExtParams;

void make_ext_params(ExtParams* params, Args* args, Styler* styler, Locked* mtNamesList);
void dest_ext_params(ExtParams* params);
