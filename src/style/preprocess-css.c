#include "preprocess-css.h"

#include "data/list.h"
#include "logs/logs.h"
#include <sass.h>
#include <stdbool.h>
#include <string.h>

static CssPreprocessResult run_scss_preprocessor(
	char** restrict data_out, char* restrict data_in, bool isSass, Str* fname, StylePreprocessorParams* params);

CssPreprocessResult preprocess_css(char** data_out, char* data_in, Str* fname, StylePreprocessorParams* params)
{
	const char* ext = strrchr(fname->str, '.');

	if (ext)
	{
		if (!strcmp(ext, ".css"))
		{
			*data_out = data_in;
			return UNPROCESSED;
		}
		else if (!strcmp(ext, ".sass") || !strcmp(ext, ".scss"))
		{
			return run_scss_preprocessor(data_out, data_in, ext[2] == 'a', fname, params);
		}
	}

	log_err("Failed to process style file: '%s' has unknown extension", fname->str);
	return FAIL;
}

static CssPreprocessResult run_scss_preprocessor(
	char** restrict data_out, char* restrict data_in, bool isSass, Str* fname, StylePreprocessorParams* params)
{
	struct Sass_Data_Context* data_ctx = sass_make_data_context(data_in);
	struct Sass_Context* ctx		   = sass_data_context_get_context(data_ctx);
	struct Sass_Options* opts		   = sass_context_get_options(ctx);

	// Set precision and output style
	if (~params->precision)
		sass_option_set_precision(opts, params->precision);
	sass_option_set_is_indented_syntax_src(opts, isSass);
	sass_option_set_output_style(opts, SASS_STYLE_COMPRESSED);

	// Construct include path from default by adding environment and command-line arguments
	/* sass_option_push_include_path(opts, "sassinc/"); */
	const char* env_sass_path = getenv("EM_STYLE_PATH");
	if (env_sass_path)
		sass_option_push_include_path(opts, env_sass_path);
	ListIter li;
	make_list_iter(&li, params->include_path);
	Str* curr;
	while (iter_list((void**)&curr, &li))
		sass_option_push_include_path(opts, curr->str);
	dest_list_iter(&li);

	// Parse SASS/SCSS
	if (sass_compile_data_context(data_ctx))
	{
		log_err("S%css compilation failed on file %s: %s", isSass ? 'a' : 'c', fname->str,
			sass_context_get_error_message(ctx));
		return FAIL;
	}

	// Obtain return value
	*data_out = sass_context_take_output_string(ctx);

	// Free memory


	return PROCESSED;
}
