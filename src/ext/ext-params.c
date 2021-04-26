#include "ext-params.h"

#include "pp/unused.h"

void init_ext_params(ExtParams* params, Args* args)
{
	params->sandbox_lvl = args->sandbox_lvl;
}

void dest_ext_params(ExtParams* params) { UNUSED(params); }
