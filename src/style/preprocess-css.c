/**
 * @file preprocess-css.c
 * @brief Provides an implementation for a CSS preprocessor using SCSS/SASS
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "preprocess-css.h"

#include "data/list.h"
#include "logs/logs.h"
#include "pp/path.h"
#include "pp/unused.h"
#include <errno.h>
#include <sass.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

typedef Sass_Import_List SassImportList;
typedef Sass_Importer_Entry SassImporterEntry;
typedef Sass_Importer_List SassImporterList;
typedef struct Sass_Compiler SassCompiler;
typedef struct Sass_Context SassContext;
typedef struct Sass_File_Context SassFileContext;
typedef struct Sass_Options SassOptions;

static int run_scss_preprocessor(char** restrict data_out, bool isSass, Str* fname, StylePreprocessorParams* params);
static int get_raw_style_file(char** restrict data_out, Str* fname, StylePreprocessorParams* params);
static void handle_style_path(SassOptions* opts, StylePreprocessorParams* params);
static void log_sass_error(SassContext* ctx, Str* fname, bool isSass);
static SassImportList importer(const char* path, SassImporterEntry cp, SassCompiler* comp);
static SassImportList trivial_importer(const char* path, SassImporterEntry cp, SassCompiler* comp);

static const char* env_style_path	 = NULL;
static bool check_for_env_style_path = true;

#ifdef _WIN32
#	define SCSS_PATH_SEP ";"
#else
#	define SCSS_PATH_SEP ":"
#endif

#define SCSS_PATH DATA_DIR SCSS_PATH_SEP "share" PATH_SEP "emblem"

int preprocess_css(char** data_out, Str* fname, StylePreprocessorParams* params)
{
	const char* ext = strrchr(fname->str, '.');

	if (ext++)
	{
		if (!strcmp(ext, "sass") || !strcmp(ext, "scss"))
			return run_scss_preprocessor(data_out, ext[1] == 'a', fname, params);
		else if (!strcmp(ext, "css"))
			return get_raw_style_file(data_out, fname, params);
	}

	log_err("Failed to process style file: '%s' has unknown extension", fname->str);
	return 1;
}

static int get_raw_style_file(char** restrict data_out, Str* fname, StylePreprocessorParams* params)
{
	int rc = 0;

	SassFileContext* file_ctx = sass_make_file_context(fname->str);
	SassContext* ctx		  = sass_file_context_get_context(file_ctx);
	SassOptions* opts		  = sass_file_context_get_options(file_ctx);

	handle_style_path(opts, params);

	// Ignore imports
	SassImporterEntry imp = sass_make_importer(trivial_importer, 0, NULL);
	SassImporterList imps = sass_make_importer_list(1);
	sass_importer_set_list_entry(imps, 0, imp);
	sass_option_set_c_importers(opts, imps);

	sass_file_context_set_options(file_ctx, opts);
	SassCompiler* compiler = sass_make_file_compiler(file_ctx);
	sass_compiler_parse(compiler);
	sass_compiler_execute(compiler);
	if (sass_context_get_error_status(ctx))
	{
		log_sass_error(ctx, fname, false);
		rc = 1;
		goto sass_cleanup;
	}

	char* file_pathed_loc = sass_compiler_find_file(fname->str, compiler);

	FILE* fp = fopen(file_pathed_loc, "r");
	if (!fp)
	{
		log_err("Failed to open '%s': %s", file_pathed_loc, strerror(errno));
		rc = 1;
		goto pathed_file_loc_cleanup;
	}

	size_t len;
	ssize_t bytes_read = getdelim(data_out, &len, '\0', fp);
	if (bytes_read == -1)
	{
		log_err("Failed to read file '%s': %s", file_pathed_loc, strerror(errno));
		rc = 1;
	}

	if (fclose(fp))
	{
		log_err("Failed to close '%s': %s", file_pathed_loc, strerror(errno));
		rc = 1;
	}

pathed_file_loc_cleanup:
	free(file_pathed_loc);
sass_cleanup:
	sass_delete_compiler(compiler);
	sass_delete_file_context(file_ctx);

	return rc;
}

static int run_scss_preprocessor(char** restrict data_out, bool isSass, Str* fname, StylePreprocessorParams* params)
{
	int rc					  = 0;
	SassFileContext* file_ctx = sass_make_file_context(fname->str);
	SassContext* ctx		  = sass_file_context_get_context(file_ctx);
	SassOptions* opts		  = sass_file_context_get_options(file_ctx);

	// Set options
	if (~params->precision)
		sass_option_set_precision(opts, params->precision);
	sass_option_set_is_indented_syntax_src(opts, isSass);

	// Set import handler
	SassImporterEntry imp = sass_make_importer(importer, 0, NULL);
	SassImporterList imps = sass_make_importer_list(1);
	sass_importer_set_list_entry(imps, 0, imp);
	sass_option_set_c_importers(opts, imps);

	// Set debugging options
	if (params->debug_sources)
	{
		sass_option_set_source_comments(opts, true);
		sass_option_set_output_style(opts, SASS_STYLE_EXPANDED);
	}
	else
		sass_option_set_output_style(opts, SASS_STYLE_COMPRESSED);

	handle_style_path(opts, params);

	sass_file_context_set_options(file_ctx, opts);

	SassCompiler* compiler = sass_make_file_compiler(file_ctx);
	sass_compiler_parse(compiler);
	sass_compiler_execute(compiler);

	*data_out = sass_context_take_output_string(ctx);
	// Retrieve errors during compilation
	if (sass_context_get_error_status(ctx))
	{
		log_sass_error(ctx, fname, isSass);
		rc = 1;
	}
	// Release memory dedicated to the C compiler
	sass_delete_compiler(compiler);
	sass_delete_file_context(file_ctx);
	return rc;
}

static void handle_style_path(SassOptions* opts, StylePreprocessorParams* params)
{
	// Construct include path from default by adding environment and command-line arguments
	if (check_for_env_style_path)
	{
		env_style_path			 = getenv("EM_STYLE_PATH");
		check_for_env_style_path = false;
	}

	if (env_style_path)
		sass_option_push_include_path(opts, env_style_path);

	ListIter li;
	make_list_iter(&li, params->include_path);
	Str* curr;
	while (iter_list((void**)&curr, &li))
		sass_option_push_include_path(opts, curr->str);
	dest_list_iter(&li);
#ifdef SCSS_PATH
	sass_option_push_include_path(opts, SCSS_PATH);
#endif
}

static SassImportList importer(const char* path, SassImporterEntry cp, SassCompiler* comp)
{
	UNUSED(cp);
	UNUSED(comp);
	log_info("Importing from path '%s'", path);
	return NULL;
}

static SassImportList trivial_importer(const char* path, SassImporterEntry cp, SassCompiler* comp)
{
	UNUSED(path);
	UNUSED(cp);
	UNUSED(comp);
	return sass_make_import_list(0);
}

static void log_sass_error(SassContext* ctx, Str* fname, bool isSass)
{
	char* msgp;
	char* msg = msgp = strdup(sass_context_get_error_message(ctx));
	while (*++msgp) { }
	msgp--;
	*msgp = '\0';
	log_err("S%css failure on file %s: %s", isSass ? 'a' : 'c', fname->str, msg);
	free(msg);
}
