/**
 * @file ext-params.c
 * @brief Implements functions to handle extension-environment parameters
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "ext-params.h"

#include "pp/unused.h"

void make_ext_params(ExtParams* params, Args* args, Styler* styler, Locked* mtNamesList)
{
	params->sandbox_lvl	  = args->sandbox_lvl;
	params->styler		  = styler;
	params->exts		  = &args->extensions;
	params->ext_args	  = &args->extension_args;
	params->args		  = args;
	params->mt_names_list = mtNamesList;
}

void dest_ext_params(ExtParams* params) { UNUSED(params); }
