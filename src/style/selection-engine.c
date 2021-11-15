/**
 * @file selection-engine.c
 * @brief Handles the selection of styles from a CSS stylesheet
 * @author Edward Jones
 * @date 2021-10-04
 */
#include "selection-engine.h"

#include "logs/logs.h"
#include "pp/path.h"
#include "pp/unused.h"
#include <libwapcaplet/libwapcaplet.h>
#include <limits.h>
#include <string.h>
#include <unistd.h>

typedef struct
{
	Str* base;
} UrlResolutionData;

static css_select_handler select_handler;
static bool caseless_isequal(lwc_string* s, lwc_string* t);

/* Function declarations required for external ops */
static css_error resolve_url(void* pw, const char* base, lwc_string* rel, lwc_string** abs);
static css_error resolve_colour(void* pw, lwc_string* name, css_color* colour);
static css_error compute_font_size(void* pw, const css_hint* parent, css_hint* size);
static bool is_http_request(const char* rel_url, size_t rel_url_len);

#define STR_LWC_EQ(s, t) caseless_isequal(get_lwc_string(s), t)

#ifdef DEBUG_CSS
#	define LOG_FUNC_NAME(...) log_debug(__VA_ARGS__)
#else
#	define LOG_FUNC_NAME(...)
#endif

int compute_style(Styler* s, DocTreeNode* node)
{
	if (!s->process_css)
		return 0;

	int rc;
	if (node->style)
		dest_style(node->style);

	StyleSelectionEngine* eng = s->engine;
	if ((rc = css_select_style(eng->ctx, node, &eng->media, NULL, &eng->handler, NULL, &node->style)))
		return rc;

	if (s->compose_styles && node->parent)
	{
		Style* new_styles = malloc(sizeof(css_select_results));
		for (int i = 0; i < CSS_PSEUDO_ELEMENT_COUNT; i++)
		{
			if (node->parent->style->styles[i] && node->style->styles[i])
			{
				rc = css_computed_style_compose(node->parent->style->styles[i], node->style->styles[i],
					compute_font_size, NULL, &new_styles->styles[i]);
				if (rc)
					return 1;
			}
			else
				new_styles->styles[i] = NULL;
		}
		dest_style(node->style);
		node->style = new_styles;
	}

	return 0;
}

int make_style_selection_engine(StyleSelectionEngine* engine)
{
	engine->handler = select_handler;
	engine->media   = (css_media) {
		.type		   = CSS_MEDIA_ALL,

		// Viewport / page media features
		.width		  = 210,
		.height	      = 297,
		.aspect_ratio = 297.0 / 210.0,
		.orientation  = CSS_MEDIA_ORIENTATION_PORTRAIT,

		// Display quality media features
		.resolution = (css_media_resolution){
		    .value = 210 * 297,
		    .unit  = CSS_UNIT_MM,
		},
		.scan            = CSS_MEDIA_SCAN_PROGRESSIVE,
		.grid            = 0,
		.update          = CSS_MEDIA_UPDATE_FREQUENCY_NORMAL,
		.overflow_block  = CSS_MEDIA_OVERFLOW_BLOCK_PAGED,
		.overflow_inline = CSS_MEDIA_OVERFLOW_INLINE_NONE,

		// Colour media features
		.color           = -1,
		.color_index     = 0, // ??
		.monochrome      = 0,
		.inverted_colors = 0,

		// Interaction media freatures
		.pointer     = CSS_MEDIA_POINTER_NONE,
		.any_pointer = CSS_MEDIA_POINTER_NONE,
		.hover       = CSS_MEDIA_HOVER_NONE,
		.any_hover   = CSS_MEDIA_HOVER_NONE,

		// Environmental media features
		.light_level = CSS_MEDIA_LIGHT_LEVEL_NORMAL,

		// Scripting media features
		.scripting = CSS_MEDIA_SCRIPTING_NONE,

		// Client details
		.client_font_size   = 16,
		.client_line_height = 100,
	};
	css_select_ctx_create(&engine->ctx);
	return 0;
}

int dest_style_selection_engine(StyleSelectionEngine* engine)
{
	if (css_select_ctx_destroy(engine->ctx))
	{
		log_err("Failed to destroy selection context");
		return 1;
	}
	return 0;
}

void make_stylesheet_params(StylesheetParams* params, Args* args)
{
	UrlResolutionData* url_res_data = malloc(sizeof(UrlResolutionData));
	url_res_data->base				= malloc(sizeof(Str));
	make_strv(url_res_data->base, "adsf");

	params->params_version = CSS_STYLESHEET_PARAMS_VERSION_1;
	params->level		   = CSS_LEVEL_3;
	params->charset		   = "UTF-8";
	params->url			   = args->style;
	params->title		   = args->style;
	params->allow_quirks   = false;
	params->inline_style   = false;
	params->resolve		   = resolve_url;
	params->resolve_pw	   = url_res_data;
	params->import		   = NULL;
	params->import_pw	   = NULL;
	params->color		   = resolve_colour;
	params->color_pw	   = NULL;
	params->font		   = NULL;
	params->font_pw		   = NULL;
}

void dest_stylesheet_params(StylesheetParams* params)
{
	UrlResolutionData* url_res_data = params->resolve_pw;
	dest_free_str(url_res_data->base);
	free(url_res_data);
}

static css_error resolve_url(void* pw, const char* base, lwc_string* rel, lwc_string** abs)
{
	LOG_FUNC_NAME("resolve_url");
	UNUSED(base);
	UrlResolutionData* url_res_data = pw;
	UNUSED(url_res_data);

	const char* rel_url		 = lwc_string_data(rel);
	const size_t rel_url_len = lwc_string_length(rel);

	// Check if absolute address must be computed.
	if ((rel_url_len && *rel_url == PATH_SEP_CHAR) || is_http_request(rel_url, rel_url_len))
		*abs = lwc_string_ref(rel);
	else
	{
		const size_t abs_url_len = PATH_MAX;
		char abs_url[PATH_MAX + 1];
		getcwd(abs_url, PATH_MAX);
		size_t path_end		= strlen(abs_url);
		abs_url[path_end++] = PATH_SEP_CHAR;
		strncpy(abs_url + path_end, rel_url, abs_url_len - path_end);

		lwc_intern_string(abs_url, strlen(abs_url), abs);
	}

	return CSS_OK;
}

static bool is_http_request(const char* rel_url, size_t rel_url_len)
{
	// Check /^https?:\/\//
	const char http_prefix[]			= "http";
	const size_t http_prefix_len		= sizeof(http_prefix) / sizeof(*http_prefix) - 1;
	const char protocol_delimiter[]		= "://";
	const size_t protocol_delimiter_len = sizeof(protocol_delimiter) / sizeof(*protocol_delimiter) - 1;

	if (rel_url_len < 7) // Check this when http/https is the only thing!
		return false;

	if (memcmp(http_prefix, rel_url, http_prefix_len))
		return false;

	const char* pdp = rel_url + http_prefix_len;
	if (*pdp == 's')
		pdp++;

	if (rel_url_len - http_prefix_len - (*pdp == 's') <= protocol_delimiter_len)
		return false;

	return !memcmp(protocol_delimiter, pdp, protocol_delimiter_len);
}

static css_error resolve_colour(void* pw, lwc_string* name, css_color* colour)
{
	UNUSED(pw);
	UNUSED(colour);
	log_warn("Unknown colour '%s'", lwc_string_data(name));
	return CSS_BADPARM;
}

static bool caseless_isequal(lwc_string* s, lwc_string* t)
{
	bool match = false;
	int rc	   = lwc_string_caseless_isequal(s, t, &match);
	if (rc != lwc_error_ok)
		log_err("lwc_string_caseless_isequal returned %d when operating on strings at %p and %p, expected lwc_error_ok "
				"= %d",
			rc, (void*)s, (void*)t, lwc_error_ok);
	return match;
}

////////////////////////////////////////////////////////////////////////////////
/// libcss interface functions                                               ///
////////////////////////////////////////////////////////////////////////////////

/* Function declarations. */
static css_error node_name(void* pw, void* node, css_qname* qname);
static css_error node_classes(void* pw, void* node, lwc_string*** classes, uint32_t* n_classes);
static css_error node_id(void* pw, void* node, lwc_string** id);
static css_error named_ancestor_node(void* pw, void* node, const css_qname* qname, void** ancestor);
static css_error named_parent_node(void* pw, void* node, const css_qname* qname, void** parent);
static css_error named_sibling_node(void* pw, void* node, const css_qname* qname, void** sibling);
static css_error named_generic_sibling_node(void* pw, void* node, const css_qname* qname, void** sibling);
static css_error parent_node(void* pw, void* node, void** parent);
static css_error sibling_node(void* pw, void* node, void** sibling);
static css_error node_has_name(void* pw, void* node, const css_qname* qname, bool* match);
static css_error node_has_class(void* pw, void* node, lwc_string* name, bool* match);
static css_error node_has_id(void* pw, void* node, lwc_string* name, bool* match);
static css_error node_has_attribute(void* pw, void* node, const css_qname* qname, bool* match);
static css_error node_has_attribute_equal(void* pw, void* node, const css_qname* qname, lwc_string* value, bool* match);
static css_error node_has_attribute_dashmatch(
	void* pw, void* node, const css_qname* qname, lwc_string* value, bool* match);
static css_error node_has_attribute_includes(
	void* pw, void* node, const css_qname* qname, lwc_string* value, bool* match);
static css_error node_has_attribute_prefix(
	void* pw, void* node, const css_qname* qname, lwc_string* value, bool* match);
static css_error node_has_attribute_suffix(
	void* pw, void* node, const css_qname* qname, lwc_string* value, bool* match);
static css_error node_has_attribute_substring(
	void* pw, void* node, const css_qname* qname, lwc_string* value, bool* match);
static css_error node_is_root(void* pw, void* node, bool* match);
static css_error node_count_siblings(void* pw, void* node, bool same_name, bool after, int32_t* count);
static css_error node_is_empty(void* pw, void* node, bool* match);
static css_error node_is_link(void* pw, void* node, bool* match);
static css_error node_is_visited(void* pw, void* node, bool* match);
static css_error node_is_hover(void* pw, void* node, bool* match);
static css_error node_is_active(void* pw, void* node, bool* match);
static css_error node_is_focus(void* pw, void* node, bool* match);
static css_error node_is_enabled(void* pw, void* node, bool* match);
static css_error node_is_disabled(void* pw, void* node, bool* match);
static css_error node_is_checked(void* pw, void* node, bool* match);
static css_error node_is_target(void* pw, void* node, bool* match);
static css_error node_is_lang(void* pw, void* node, lwc_string* lang, bool* match);
static css_error node_presentational_hint(void* pw, void* node, uint32_t* nhints, css_hint** hints);
static css_error ua_default_for_property(void* pw, uint32_t property, css_hint* hint);
static css_error set_libcss_node_data(void* pw, void* n, void* libcss_node_data);
static css_error get_libcss_node_data(void* pw, void* n, void** libcss_node_data);

/* static css_unit_ctx uint_len_ctx = { */
/* .viewport_width	   = 800 * (1 << CSS_RADIX_POINT), */
/* .viewport_height   = 600 * (1 << CSS_RADIX_POINT), */
/* .font_size_default = 16 * (1 << CSS_RADIX_POINT), */
/* .font_size_minimum = 6 * (1 << CSS_RADIX_POINT), */
/* .device_dpi		   = 96 * (1 << CSS_RADIX_POINT), */
/* .root_style		   = NULL, [> We don't have a root node yet. <] */
/* .pw				   = NULL, [> We're not implementing measure callback <] */
/* .measure		   = NULL, [> We're not implementing measure callback <] */
/* }; */

/* Table of function pointers for the LibCSS Select API. */
static css_select_handler select_handler = {
	.handler_version			  = CSS_SELECT_HANDLER_VERSION_1,
	.node_name					  = node_name,
	.node_classes				  = node_classes,
	.node_id					  = node_id,
	.named_ancestor_node		  = named_ancestor_node,
	.named_parent_node			  = named_parent_node,
	.named_sibling_node			  = named_sibling_node,
	.named_generic_sibling_node	  = named_generic_sibling_node,
	.parent_node				  = parent_node,
	.sibling_node				  = sibling_node,
	.node_has_name				  = node_has_name,
	.node_has_class				  = node_has_class,
	.node_has_id				  = node_has_id,
	.node_has_attribute			  = node_has_attribute,
	.node_has_attribute_equal	  = node_has_attribute_equal,
	.node_has_attribute_dashmatch = node_has_attribute_dashmatch,
	.node_has_attribute_includes  = node_has_attribute_includes,
	.node_has_attribute_prefix	  = node_has_attribute_prefix,
	.node_has_attribute_suffix	  = node_has_attribute_suffix,
	.node_has_attribute_substring = node_has_attribute_substring,
	.node_is_root				  = node_is_root,
	.node_count_siblings		  = node_count_siblings,
	.node_is_empty				  = node_is_empty,
	.node_is_link				  = node_is_link,
	.node_is_visited			  = node_is_visited,
	.node_is_hover				  = node_is_hover,
	.node_is_active				  = node_is_active,
	.node_is_focus				  = node_is_focus,
	.node_is_enabled			  = node_is_enabled,
	.node_is_disabled			  = node_is_disabled,
	.node_is_checked			  = node_is_checked,
	.node_is_target				  = node_is_target,
	.node_is_lang				  = node_is_lang,
	.node_presentational_hint	  = node_presentational_hint,
	.ua_default_for_property	  = ua_default_for_property,
	.compute_font_size			  = compute_font_size,
	.set_libcss_node_data		  = set_libcss_node_data,
	.get_libcss_node_data		  = get_libcss_node_data,
};

static css_error node_name(void* pw, void* n, css_qname* qname) // Done
{
	LOG_FUNC_NAME("node_name");
	DocTreeNode* node = n;
	UNUSED(pw);

	qname->name = lwc_string_ref(get_lwc_string(node->style_name));
	qname->ns	= NULL;

	return CSS_OK;
}

static css_error node_classes(void* pw, void* n, lwc_string*** classes, uint32_t* n_classes) // Check
{
	LOG_FUNC_NAME("node_classes");
	UNUSED(pw);
	DocTreeNode* node = n;
	*classes		  = node->style_data->classes;
	*n_classes		  = node->style_data->n_classes;
	lwc_string_ref(**classes);
	return CSS_OK;
}

static css_error node_id(void* pw, void* n, lwc_string** id) // IDs are not supported
{
	LOG_FUNC_NAME("node_id");
	UNUSED(pw);
	UNUSED(n);
	*id = NULL;
	return CSS_OK;
}

static css_error named_ancestor_node(
	void* pw, void* n, const css_qname* qname, void** ancestor) // Get ancestor only if it has a particular name
{
	LOG_FUNC_NAME("named_ancestor_node");
	UNUSED(pw);
	DocTreeNode* node  = n;
	DocTreeNode* anode = node->parent;
	bool match = false;
	while (anode && !(match = STR_LWC_EQ(anode->style_name, qname->name)))
		anode = anode->parent;
	*ancestor = match ? anode : NULL;
	return CSS_OK;
}

static css_error named_parent_node(
	void* pw, void* n, const css_qname* qname, void** parent) // Only return the parent if it has the correct name
{
	LOG_FUNC_NAME("named_parent_node");
	UNUSED(pw);
	DocTreeNode* node = n;
	if (node->parent && STR_LWC_EQ(node->style_name, qname->name))
		*parent = node->parent;
	else
		*parent = NULL;
	return CSS_OK;
}

static css_error named_generic_sibling_node(void* pw, void* n, const css_qname* qname, void** sibling) // Done
{
	LOG_FUNC_NAME("named_generic_sibling_node");
	UNUSED(pw);
	DocTreeNode* node = n;
	DocTreeNode* sib  = node->prev_sibling;
	while (sib && !STR_LWC_EQ(node->style_name, qname->name))
		sib = sib->prev_sibling;
	*sibling = sib;
	return CSS_OK;
}

static css_error named_sibling_node(void* pw, void* n, const css_qname* qname,
	void** sibling) // Done
{
	LOG_FUNC_NAME("named_sibling_node");
	UNUSED(pw);
	DocTreeNode* node = n;
	if (node->prev_sibling && STR_LWC_EQ(node->prev_sibling->style_name, qname->name))
		*sibling = node->prev_sibling;
	else
		*sibling = NULL;
	return CSS_OK;
}

static css_error parent_node(void* pw, void* n, void** parent) // Done
{
	UNUSED(pw);
	DocTreeNode* node = n;
	*parent			  = node->parent;
	LOG_FUNC_NAME(
		"parent_node %s |-> %s", node->style_name->str, node->parent ? node->parent->style_name->str : "(n/a)");
	return CSS_OK;
}

static css_error sibling_node(void* pw, void* n, void** sibling) // Done
{
	LOG_FUNC_NAME("sibling_node");
	UNUSED(pw);
	DocTreeNode* node = n;
	*sibling		  = node->prev_sibling;
	return CSS_OK;
}

static css_error node_has_name(void* pw, void* n, const css_qname* qname, bool* match) // Done
{
	LOG_FUNC_NAME("node_has_name");
	UNUSED(pw);
	DocTreeNode* node = n;
	assert(lwc_string_caseless_isequal(get_lwc_string(node->style_name), qname->name, match) == lwc_error_ok);
	return CSS_OK;
}

static css_error node_has_class(void* pw, void* n, lwc_string* name, bool* match) // Done
{
	LOG_FUNC_NAME("node_has_class");
	UNUSED(pw);
	DocTreeNode* node = n;
	assert(lwc_string_caseless_isequal(get_lwc_string(node->style_name), name, match) == lwc_error_ok);
	return CSS_OK;
}

static css_error node_has_id(void* pw, void* n, lwc_string* name, bool* match) // IDs are not supported
{
	LOG_FUNC_NAME("node_has_id");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(name);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute(
	void* pw, void* n, const css_qname* qname, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute_equal(
	void* pw, void* n, const css_qname* qname, lwc_string* value, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute_equal");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	UNUSED(value);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute_dashmatch(
	void* pw, void* n, const css_qname* qname, lwc_string* value, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute_dashmatch");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	UNUSED(value);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute_includes(
	void* pw, void* n, const css_qname* qname, lwc_string* value, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute_includes");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	UNUSED(value);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute_prefix(
	void* pw, void* n, const css_qname* qname, lwc_string* value, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute_prefix");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	UNUSED(value);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute_suffix(
	void* pw, void* n, const css_qname* qname, lwc_string* value, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute_suffix");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	UNUSED(value);
	*match = false;
	return CSS_OK;
}

static css_error node_has_attribute_substring(
	void* pw, void* n, const css_qname* qname, lwc_string* value, bool* match) // Attributes are not supported
{
	LOG_FUNC_NAME("node_has_attribute_substring");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(qname);
	UNUSED(value);
	*match = false;
	return CSS_OK;
}

static css_error node_is_root(void* pw, void* n, bool* match) // Done
{
	LOG_FUNC_NAME("node_is_root");
	UNUSED(pw);
	DocTreeNode* node = n;
	*match			  = !node->parent;
	return CSS_OK;
}

static css_error node_count_siblings(
	void* pw, void* n, bool same_name, bool after, int32_t* count) // What are same_name and after for??
{
	LOG_FUNC_NAME("node_count_siblings");
	// Includes self
	UNUSED(pw);
	UNUSED(same_name);
	UNUSED(after);
	DocTreeNode* node	= n;
	DocTreeNode* parent = node->parent;
	if (parent)
		switch (parent->content->type)
		{
			case CONTENT:
				*count = parent->content->content->cnt;
				break;
			case WORD:
			case CALL:
				*count = 1;
		}
	else
		*count = 1; // The root has no siblings
	return CSS_OK;
}

static css_error node_is_empty(void* pw, void* n, bool* match) // Done
{
	LOG_FUNC_NAME("node_is_empty");
	UNUSED(pw);
	DocTreeNode* node = n;
	switch (node->content->type)
	{
		case WORD:
			*match = false;
			break;
		case CALL:
			*match = !node->content->call->result;
			break;
		case CONTENT:
			*match = !node->content->content->cnt;
			break;
	}
	*match = false;
	return CSS_OK;
}

static css_error node_is_link(void* pw, void* n, bool* match) // Links are not supported
{
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_visited(void* pw, void* n, bool* match) // Links are not supported
{
	LOG_FUNC_NAME("node_is_visited");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_hover(void* pw, void* n, bool* match) // Links are not supported
{
	LOG_FUNC_NAME("node_is_hover");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_active(void* pw, void* n, bool* match) // Links are not supported
{
	LOG_FUNC_NAME("node_is_active");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_focus(void* pw, void* n, bool* match) // Links are not supported
{
	LOG_FUNC_NAME("node_is_focus");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_enabled(void* pw, void* n, bool* match) // Node enabling is not supported
{
	LOG_FUNC_NAME("node_is_enabled");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_disabled(void* pw, void* n, bool* match) // Node enabling is not supported
{
	LOG_FUNC_NAME("node_is_disabled");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_checked(void* pw, void* n, bool* match) // Interactive nodes are not supported
{
	LOG_FUNC_NAME("node_is_checked");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_target(void* pw, void* n, bool* match) // Targets are not supported.
{
	LOG_FUNC_NAME("node_is_target");
	UNUSED(pw);
	UNUSED(n);
	*match = false;
	return CSS_OK;
}

static css_error node_is_lang(void* pw, void* n, lwc_string* lang, bool* match) // Lang is not supported
{
	LOG_FUNC_NAME("node_is_lang");
	UNUSED(pw);
	UNUSED(n);
	UNUSED(lang);
	*match = false;
	return CSS_OK;
}

static css_error node_presentational_hint(void* pw, void* node, uint32_t* nhints, css_hint** hints)
{
	LOG_FUNC_NAME("node_presentational_hint");
	UNUSED(pw);
	UNUSED(node);
	*nhints = 0;
	*hints	= NULL;
	return CSS_OK;
}

static css_error ua_default_for_property(void* pw, uint32_t property, css_hint* hint)
{
	LOG_FUNC_NAME("ua_default_for_property");
	UNUSED(pw);

	if (property == CSS_PROP_COLOR)
	{
		hint->data.color = 0x00000000;
		hint->status	 = CSS_COLOR_COLOR;
	}
	else if (property == CSS_PROP_FONT_FAMILY)
	{
		hint->data.strings = NULL;
		hint->status	   = CSS_FONT_FAMILY_SANS_SERIF;
	}
	else if (property == CSS_PROP_QUOTES)
	{
		/* Not exactly useful :) */
		hint->data.strings = NULL;
		hint->status	   = CSS_QUOTES_NONE;
	}
	else if (property == CSS_PROP_VOICE_FAMILY)
	{
		/* Voice family is not supported. */
		hint->data.strings = NULL;
		hint->status	   = 0;
	}
	else
	{
		log_err("Invalid property to get default: %d", property);
		return CSS_INVALID;
	}

	return CSS_OK;
}

static css_error compute_font_size(void* pw, const css_hint* parent, css_hint* size)
{
	LOG_FUNC_NAME("compute_font_size");
	UNUSED(pw);
	size->data.integer = parent ? parent->data.integer : 16;
	size->status	   = 0;
	return CSS_OK;
}

static css_error set_libcss_node_data(void* pw, void* n, void* libcss_node_data)
{
	LOG_FUNC_NAME("set_libcss_node_data");
	UNUSED(pw);

	DocTreeNode* node = n;

	// Prevent leak
	int rc;
	if (node->style_data->node_css_data != libcss_node_data)
		if ((rc = modify_node_data(node, NODE_DATA_DELETED)))
			return rc;

	// Store new data
	node->style_data->node_css_data = libcss_node_data;

	return CSS_OK;
}

css_error modify_node_data(DocTreeNode* node, NodeDataAction action)
{
	LOG_FUNC_NAME("modify_node_data");
	if (action == NODE_DATA_DELETED && !node->style_data->node_css_data)
		return CSS_OK;
	return css_libcss_node_data_handler(
		&select_handler, (css_node_data_action)action, NULL, node, NULL, node->style_data->node_css_data);
}

static css_error get_libcss_node_data(void* pw, void* n, void** libcss_node_data)
{
	LOG_FUNC_NAME("get_libcss_node_data");
	UNUSED(pw);
	DocTreeNode* node = n;
	*libcss_node_data = node->style_data->node_css_data;

	return CSS_OK;
}
