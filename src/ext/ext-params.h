#pragma once

#include "argp.h"
#include "style/css-params.h"
#include <lua.h>
#include <stdbool.h>

typedef struct
{
	int sandbox_lvl;
	Styler* styler;
} ExtParams;

void init_ext_params(ExtParams* params, Args* args, Styler* styler);
void dest_ext_params(ExtParams* params);
