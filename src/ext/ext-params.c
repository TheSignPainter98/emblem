#include "ext-params.h"

#include "pp/unused.h"

void init_ext_params(ExtParams* params, Args* args, Styler* styler)
{
	params->sandbox_lvl = args->sandbox_lvl;
	params->styler		= styler;
	params->exts		= &args->extensions;
	params->ext_args	= &args->extension_args;
}

void dest_ext_params(ExtParams* params) { UNUSED(params); }
