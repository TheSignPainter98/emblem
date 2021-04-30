#pragma once

#include "argp.h"
#include "data/list.h"
#include "data/str.h"
#include <libcss/libcss.h>

struct StylePreprocessorParams_s;

typedef struct
{
	css_stylesheet* stylesheet;
	Str* user_style_file;
	struct StylePreprocessorParams_s* prep_params;
	Str* default_typeface;
	double default_font_size;
	List* snippets;
} Styler;
typedef css_select_results Style;

typedef struct StylePreprocessorParams_s
{
	int precision;
	List* include_path;
} StylePreprocessorParams;

void make_style_preprocessor_params(StylePreprocessorParams* params, Args* args);
void dest_style_preprocessor_params(StylePreprocessorParams* params);
