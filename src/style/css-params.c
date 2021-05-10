#include "css-params.h"

#include "pp/lambda.h"
#include "pp/unused.h"

void make_style_preprocessor_params(StylePreprocessorParams* params, Args* args)
{
	UNUSED(args);
	params->precision	 = 7;
	params->include_path = malloc(sizeof(List));
	make_list(params->include_path);
}

void dest_style_preprocessor_params(StylePreprocessorParams* params)
{
	NON_ISO(Destructor ed = (Destructor)ilambda(void, (Str * s), {
		dest_str(s);
		free(s);
	}));
	dest_list(params->include_path, true, ed);
	free(params->include_path);
}
