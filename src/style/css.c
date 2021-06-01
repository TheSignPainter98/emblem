#include "css.h"

#include "argp.h"
#include "logs/logs.h"
#include "pp/unused.h"
#include "preprocess-css.h"
#include <errno.h>
#include <libcss/libcss.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define DEFAULT_STYLESHEET_EXTENSION	".scss"
#define USER_STYLE_OVERRIDE_FONT_FAMILY ".body{font-family:'%s';}"
#define USER_STYLE_OVERRIDE_FONT_SIZE	".body{font-size:%f;}"

static int append_user_style_overrides(Styler* styler);
static css_error resolve_url(void* pw, const char* base, lwc_string* rel, lwc_string** abs);

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

	// code = css_stylesheet_create(&params, myrealloc, NULL, &sheet);
	css_stylesheet_params params;
	params.params_version = CSS_STYLESHEET_PARAMS_VERSION_1;
	params.level		  = CSS_LEVEL_3;
	params.charset		  = "UTF-8";
	params.url			  = args->style;
	params.title		  = args->style;
	params.allow_quirks	  = false;
	params.inline_style	  = false;
	params.resolve		  = resolve_url;
	params.resolve_pw	  = NULL;
	params.import		  = NULL;
	params.import_pw	  = NULL;
	params.color		  = NULL;
	params.color_pw		  = NULL;
	params.font			  = NULL;
	params.font_pw		  = NULL;
	styler->stylesheet	  = NULL;
	int rc				  = css_stylesheet_create(&params, &styler->stylesheet);
	if (rc != CSS_OK)
	{
		log_err("Failed to create css stylesheet");
		exit(1);
	}

	styler->prep_params = malloc(sizeof(StylePreprocessorParams));
	make_style_preprocessor_params(styler->prep_params, args);
}

void dest_styler(Styler* styler)
{
	dest_str(styler->default_typeface);
	free(styler->default_typeface);
	dest_list(styler->snippets, true, (Destructor)dest_free_str);
	/* NON_ISO(dest_list(styler->snippets, true, (Destructor)dest_str)); */
	free(styler->snippets);
	dest_style_preprocessor_params(styler->prep_params);
	free(styler->prep_params);
	dest_str(styler->user_style_file);
	free(styler->user_style_file);
	css_stylesheet_destroy(styler->stylesheet);
}

int prepare_styler(Styler* styler)
{
	int rc;
	rc = append_style_sheet(styler, styler->user_style_file);
	if (rc != CSS_OK)
		return 1;
	rc = append_user_style_overrides(styler);
	if (rc)
		return 1;
	rc = css_stylesheet_data_done(styler->stylesheet);
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

int append_style_sheet(Styler* styler, Str* sheet_loc)
{
	log_debug("Appending stylesheet %s", sheet_loc->str);
	if (access(sheet_loc->str, R_OK))
	{
		log_err("Could not read file '%s': %s", sheet_loc->str, strerror(errno));
		return 1;
	}

	// Open file
	FILE* fp = fopen(sheet_loc->str, "r");
	if (!fp)
	{
		log_err("Failed to open file '%s': %s", sheet_loc->str, strerror(errno));
		return 1;
	}

	// Read file
	fseek(fp, 0, SEEK_END);
	size_t len = ftell(fp);
	fseek(fp, 0, SEEK_SET);
	char* raw_stylesheet_content = malloc(1 + len);
	size_t fr					 = fread(raw_stylesheet_content, 1, len, fp);
	raw_stylesheet_content[len]	 = '\0';
	if (fr != len)
	{
		if (feof(fp))
			log_err("Premature end of file detected while reading %s", sheet_loc->str);
		else if (ferror(fp))
			log_err("Error while reading %s: %zu", sheet_loc->str, fr);
		exit(1);
	}

	if (fclose(fp))
	{
		log_err("Failed to close file after reading '%s': %s", sheet_loc->str, strerror(errno));
		return 1;
	}

	char* preprocessed_stylesheet_content = NULL;
	CssPreprocessResult prs
		= preprocess_css(&preprocessed_stylesheet_content, raw_stylesheet_content, sheet_loc, styler->prep_params);
	if (prs == FAIL)
		return 1;
	Str* preprocessed_stylesheet_content_str = malloc(sizeof(Str));
	make_strr(preprocessed_stylesheet_content_str, preprocessed_stylesheet_content);
	ListNode* ln = malloc(sizeof(ListNode));
	make_list_node(ln, preprocessed_stylesheet_content_str);
	append_list_node(styler->snippets, ln);

	// Append style to sheet
	css_error rc = css_stylesheet_append_data(styler->stylesheet,
		(const uint8_t*)preprocessed_stylesheet_content_str->str, preprocessed_stylesheet_content_str->len);
	if (rc != CSS_OK && rc != CSS_NEEDDATA)
	{
		log_err("Failed to append stylesheet to styler: %d", rc);
		return 1;
	}

	return 0;
}

void dest_style(Style* style) { css_select_results_destroy(style); }

css_error resolve_url(void* pw, const char* base, lwc_string* rel, lwc_string** abs)
{
	UNUSED(pw);
	UNUSED(base);

	/* About as useless as possible */
	*abs = lwc_string_ref(rel);

	return CSS_OK;
}
