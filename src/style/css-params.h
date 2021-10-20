/**
 * @file css-params.h
 * @brief Exposes functions to handle CSS-environment parameters
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "argp.h"
#include "data/list.h"
#include "data/str.h"
#include <libcss/libcss.h>

struct StylePreprocessorParams_s;
struct StyleSelectionEngine_s;

typedef css_stylesheet_params StylesheetParams;

typedef struct
{
	css_stylesheet* stylesheet;
	StylesheetParams* stylesheet_params;
	struct StyleSelectionEngine_s* engine;
	Str* user_style_file;
	struct StylePreprocessorParams_s* prep_params;
	Str* default_typeface;
	double default_font_size;
	List* snippets;
} Styler;

typedef struct StyleSelectionEngine_s
{
	css_media media;
	css_select_handler handler;
	css_select_ctx* ctx;
} StyleSelectionEngine;

typedef struct StylePreprocessorParams_s
{
	int precision;
	List* include_path;
} StylePreprocessorParams;

typedef css_select_results Style;

typedef struct
{
	lwc_string** classes;
	int n_classes;
} StyleData;

void make_style_preprocessor_params(StylePreprocessorParams* params, Args* args);
void dest_style_preprocessor_params(StylePreprocessorParams* params);
