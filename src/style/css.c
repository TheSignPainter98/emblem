/**
 * @file css.c
 * @brief Manages resolution of styles and the handling of stylesheets
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "css.h"

#include "argp.h"
#include "ext/style.h"
#include "logs/logs.h"
#include "pp/unused.h"
#include "preprocess-css.h"
#include "selection-engine.h"
#include <errno.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>

#ifdef DEBUG_CSS
#	include <stdio.h>
#endif

#define DEFAULT_STYLESHEET_EXTENSION	".scss"
#define USER_STYLE_OVERRIDE_FONT_FAMILY ".body{font-family:'%s';}"
#define USER_STYLE_OVERRIDE_FONT_SIZE	".body{font-size:%f;}"

static int append_user_style_overrides(Styler* styler);

void make_styler(Styler* styler, Args* args)
{
	styler->default_font_size = args->default_font_size;
	styler->default_typeface  = malloc(sizeof(Str));
	make_strc(styler->default_typeface, args->default_typeface);
	styler->snippets = malloc(sizeof(List));
	make_list(styler->snippets);
	styler->user_style_file = malloc(sizeof(Str));
	const char* ext			= strrchr(args->style, '.');
	if (!ext)
	{
		const size_t style_loc_len = strlen(args->style);
		char style_loc[style_loc_len + sizeof(DEFAULT_STYLESHEET_EXTENSION) + 1];
		memcpy(style_loc, args->style, strlen(args->style));
		strcpy(style_loc + style_loc_len, DEFAULT_STYLESHEET_EXTENSION); // NOLINT
		log_debug("Reading from implied file %s", style_loc);
		make_strc(styler->user_style_file, style_loc);
	}
	else
		make_strc(styler->user_style_file, args->style);

	styler->stylesheet_params = malloc(sizeof(css_stylesheet_params));
	make_stylesheet_params(styler->stylesheet_params, args);
	int rc = css_stylesheet_create(styler->stylesheet_params, &styler->stylesheet);
	if (rc != CSS_OK)
	{
		log_err("Failed to create css stylesheet");
		exit(1);
	}

	styler->engine = malloc(sizeof(StyleSelectionEngine));
	make_style_selection_engine(styler->engine);

	styler->prep_params = malloc(sizeof(StylePreprocessorParams));
	make_style_preprocessor_params(styler->prep_params, args);

	// These get filled in once the output driver has been loaded
	styler->process_css = false;
	styler->process_scss = false;
}

void dest_styler(Styler* styler)
{
	dest_str(styler->default_typeface);
	free(styler->default_typeface);
	dest_list(styler->snippets, (Destructor)dest_free_str);
	free(styler->snippets);
	dest_stylesheet_params(styler->stylesheet_params);
	free(styler->stylesheet_params);
	dest_style_selection_engine(styler->engine);
	free(styler->engine);
	dest_style_preprocessor_params(styler->prep_params);
	free(styler->prep_params);
	dest_str(styler->user_style_file);
	free(styler->user_style_file);
	if (css_stylesheet_destroy(styler->stylesheet))
		log_err("Failed to destroy css stylesheet");
}

int prepare_styler(Styler* styler, ExtensionState* s)
{
	int rc;
	if (styler->process_scss)
	{
		rc = import_stylesheets_from_extensions(s, styler, styler->process_css);
		if (rc)
			return rc;
		if (styler->process_css)
		{
			rc = append_style_sheet(styler, styler->user_style_file, NULL, true);
			if (rc != CSS_OK)
				return 1;
		}
	}
	rc = append_user_style_overrides(styler);
	if (rc)
		return 1;
	rc = css_stylesheet_data_done(styler->stylesheet);
	if (rc != CSS_OK)
		return 1;
	rc = css_select_ctx_append_sheet(styler->engine->ctx, styler->stylesheet, CSS_ORIGIN_AUTHOR, NULL);
	if (rc != CSS_OK)
		return 1;
	return 0;
}

static int append_user_style_overrides(Styler* styler)
{
	log_debug("Applying user-style overrides...");
	bool ffdef	 = *styler->default_typeface->str;
	bool fsdef	 = styler->default_font_size != 0;
	size_t fflen = 1 + (ffdef ? snprintf(NULL, 0, USER_STYLE_OVERRIDE_FONT_FAMILY, styler->default_typeface->str) : 0);
	size_t fslen = 1 + (fsdef ? snprintf(NULL, 0, USER_STYLE_OVERRIDE_FONT_SIZE, styler->default_font_size) : 0);

	if (ffdef || fsdef)
	{
		char user_style_override[fflen + fslen];
		if (ffdef)
			snprintf(user_style_override, fflen, USER_STYLE_OVERRIDE_FONT_FAMILY, styler->default_typeface->str);
		if (fsdef)
			snprintf(user_style_override + fflen - 1, fslen, USER_STYLE_OVERRIDE_FONT_SIZE, styler->default_font_size);

		css_error rc
			= css_stylesheet_append_data(styler->stylesheet, (const uint8_t*)user_style_override, fflen + fslen - 1);
		if (rc != CSS_OK && rc != CSS_NEEDDATA)
		{
			log_err("Failed to append stylesheet to styler: %d", rc);
			return 1;
		}
	}
	return 0;
}

int append_style_sheet(Styler* styler, Str* sheet_loc, Str* sheet_data, bool append_css_to_context)
{
	log_debug("Appending stylesheet '%s' (%s)", sheet_loc->str, sheet_data ? "internal" : "external");

	char* preprocessed_stylesheet_content = NULL;
	int prs = preprocess_css(&preprocessed_stylesheet_content, sheet_loc, sheet_data, styler->prep_params);
	if (prs)
		return prs;

#ifdef DEBUG_SASS
	fputs(preprocessed_stylesheet_content, stderr);
#endif

	Str* preprocessed_stylesheet_content_str = malloc(sizeof(Str));
	make_strr(preprocessed_stylesheet_content_str, preprocessed_stylesheet_content);
	append_list(styler->snippets, preprocessed_stylesheet_content_str);

	// Append style to sheet
	if (append_css_to_context)
	{
		css_error rc = css_stylesheet_append_data(styler->stylesheet,
			(const uint8_t*)preprocessed_stylesheet_content_str->str, preprocessed_stylesheet_content_str->len);
		if (rc != CSS_OK && rc != CSS_NEEDDATA)
		{
			log_err("Failed to append stylesheet to styler: %d", rc);
			return 1;
		}
	}
	return 0;
}

void make_style(Style* style) { UNUSED(style); }

void dest_style(Style* style) { css_select_results_destroy(style); }

void make_style_data(StyleData* data, Str* style_name, DocTreeNode* node)
{
	data->n_classes		= 1;
	data->classes		= malloc(sizeof(lwc_string*));
	*data->classes		= lwc_string_ref(get_lwc_string(style_name));
	data->node			= node;
	data->node_css_data = NULL;
}

void dest_style_data(StyleData* data)
{
	for (int i = 0; i < data->n_classes; i++)
	{
		lwc_string_unref(data->classes[i]);
	}
	free(data->classes);

	if (data->node_css_data)
		modify_node_data(data->node, NODE_DATA_DELETED);
}
