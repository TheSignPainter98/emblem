/**
 * @file css-params.c
 * @brief Implements functions to handle CSS-environment parameters
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "css-params.h"

#include "data/dest-free.h"
#include "pp/lambda.h"
#include "pp/unused.h"

#define SASS_PREPROC_DEFAULT_PRECISION 7

void make_style_preprocessor_params(StylePreprocessorParams* params, Args* args)
{
	UNUSED(args);
	params->precision	  = SASS_PREPROC_DEFAULT_PRECISION;
	params->debug_sources = args->debug_scss_sources;
	params->include_path  = malloc(sizeof(List));
	make_list(params->include_path);
}

void dest_style_preprocessor_params(StylePreprocessorParams* params)
{
	dest_list(params->include_path, (Destructor)dest_free_str);
	free(params->include_path);
}
