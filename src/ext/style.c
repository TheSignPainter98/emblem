/**
 * @file style.c
 * @brief Implements function for loading stylesheets from extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "style.h"

#include "data/str.h"
#include "debug.h"
#include "ext-env.h"
#include "logs/logs.h"
#include "lua-ast-io.h"
#include "lua.h"
#include "pp/assert.h"
#include "pp/unused.h"
#include "style/css.h"
#include <lauxlib.h>
#include <libcss/fpmath.h>
#include <stdbool.h>
#include <string.h>

#define EM_IMPORT_STYLESHEET_FUNC_NAME "stylesheet"
#define STYLESHEET_LIST_RIDX		   "emblem_stylesheets"

#define STYLE_PACKER_DECL(field) register_api_function(s, "__get_" #field, ext_get_##field);
#define STYLE_PACKER_SIG(field)	 static int ext_get_##field(ExtensionState* s)

static const char* const pseudo_element_names[] = {
	[CSS_PSEUDO_ELEMENT_NONE]		  = NULL, // Unused
	[CSS_PSEUDO_ELEMENT_FIRST_LINE]	  = "first_line",
	[CSS_PSEUDO_ELEMENT_FIRST_LETTER] = "first_letter",
	[CSS_PSEUDO_ELEMENT_BEFORE]		  = "before",
	[CSS_PSEUDO_ELEMENT_AFTER]		  = "after",
};

static const int num_style_elements = 59;

static int ext_declare_stylesheet(ExtensionState* s);
static int pack_content(ExtensionState* s, const css_computed_content_item* content, DocTreeNode* node);

STYLE_PACKER_SIG(colour);
STYLE_PACKER_SIG(top);
STYLE_PACKER_SIG(bottom);
STYLE_PACKER_SIG(left);
STYLE_PACKER_SIG(right);
STYLE_PACKER_SIG(background);
STYLE_PACKER_SIG(border_bottom);
STYLE_PACKER_SIG(border_left);
STYLE_PACKER_SIG(border_right);
STYLE_PACKER_SIG(border_top);
STYLE_PACKER_SIG(column_rule);
STYLE_PACKER_SIG(outline);
STYLE_PACKER_SIG(colour);
STYLE_PACKER_SIG(clip);
STYLE_PACKER_SIG(flex_grow);
STYLE_PACKER_SIG(flex_shrink);
STYLE_PACKER_SIG(opacity);
STYLE_PACKER_SIG(column_count);
STYLE_PACKER_SIG(order);
STYLE_PACKER_SIG(orphans);
STYLE_PACKER_SIG(widows);
STYLE_PACKER_SIG(z_index);
STYLE_PACKER_SIG(column_gap);
STYLE_PACKER_SIG(column_width);
STYLE_PACKER_SIG(flex_basis);
STYLE_PACKER_SIG(height);
STYLE_PACKER_SIG(letter_spacing);
STYLE_PACKER_SIG(margin_bottom);
STYLE_PACKER_SIG(margin_left);
STYLE_PACKER_SIG(margin_right);
STYLE_PACKER_SIG(margin_top);
STYLE_PACKER_SIG(max_height);
STYLE_PACKER_SIG(max_width);
STYLE_PACKER_SIG(min_height);
STYLE_PACKER_SIG(min_width);
STYLE_PACKER_SIG(padding_bottom);
STYLE_PACKER_SIG(padding_left);
STYLE_PACKER_SIG(padding_right);
STYLE_PACKER_SIG(padding_top);
STYLE_PACKER_SIG(text_indent);
STYLE_PACKER_SIG(vertical_align);
STYLE_PACKER_SIG(width);
STYLE_PACKER_SIG(word_spacing);
STYLE_PACKER_SIG(border_width_bottom);
STYLE_PACKER_SIG(border_width_left);
STYLE_PACKER_SIG(border_width_right);
STYLE_PACKER_SIG(border_width_top);
STYLE_PACKER_SIG(column_rule_width);
STYLE_PACKER_SIG(font_size);
STYLE_PACKER_SIG(line_height);
STYLE_PACKER_SIG(outline_width);
STYLE_PACKER_SIG(border_spacing);
STYLE_PACKER_SIG(background_position);
STYLE_PACKER_SIG(align_content);
STYLE_PACKER_SIG(align_items);
STYLE_PACKER_SIG(align_self);
STYLE_PACKER_SIG(border_collapse);
STYLE_PACKER_SIG(border_bottom_style);
STYLE_PACKER_SIG(border_left_style);
STYLE_PACKER_SIG(border_right_style);
STYLE_PACKER_SIG(border_top_style);
STYLE_PACKER_SIG(box_sizing);
STYLE_PACKER_SIG(break_after);
STYLE_PACKER_SIG(break_before);
STYLE_PACKER_SIG(break_inside);
STYLE_PACKER_SIG(caption_side);
STYLE_PACKER_SIG(clear);
STYLE_PACKER_SIG(column_fill);
STYLE_PACKER_SIG(column_rule_style);
STYLE_PACKER_SIG(column_span);
STYLE_PACKER_SIG(direction);
STYLE_PACKER_SIG(display_static);
STYLE_PACKER_SIG(empty_cells);
STYLE_PACKER_SIG(flex_direction);
STYLE_PACKER_SIG(flex_wrap);
STYLE_PACKER_SIG(float);
STYLE_PACKER_SIG(font_style);
STYLE_PACKER_SIG(font_variant);
STYLE_PACKER_SIG(font_weight);
STYLE_PACKER_SIG(justify_content);
STYLE_PACKER_SIG(list_style_position);
STYLE_PACKER_SIG(list_style_type);
STYLE_PACKER_SIG(outline_style);
STYLE_PACKER_SIG(overflow_x);
STYLE_PACKER_SIG(overflow_y);
STYLE_PACKER_SIG(page_break_after);
STYLE_PACKER_SIG(page_break_before);
STYLE_PACKER_SIG(page_break_inside);
STYLE_PACKER_SIG(position);
STYLE_PACKER_SIG(table_layout);
STYLE_PACKER_SIG(text_align);
STYLE_PACKER_SIG(text_decoration);
STYLE_PACKER_SIG(text_transform);
STYLE_PACKER_SIG(unicode_bidi);
STYLE_PACKER_SIG(visibility);
STYLE_PACKER_SIG(white_space);
STYLE_PACKER_SIG(writing_mode);
STYLE_PACKER_SIG(background_attachment);
STYLE_PACKER_SIG(display);
STYLE_PACKER_SIG(content);
STYLE_PACKER_SIG(font_family);

void register_ext_style(ExtensionState* s)
{
	lua_newtable(s);
	lua_setfield(s, LUA_REGISTRYINDEX, STYLESHEET_LIST_RIDX);

	register_api_function(s, EM_IMPORT_STYLESHEET_FUNC_NAME, ext_declare_stylesheet);
	register_api_table(s, "__css", {
		STYLE_PACKER_DECL(colour);
		STYLE_PACKER_DECL(top);
		STYLE_PACKER_DECL(bottom);
		STYLE_PACKER_DECL(left);
		STYLE_PACKER_DECL(right);
		STYLE_PACKER_DECL(background);
		STYLE_PACKER_DECL(border_bottom);
		STYLE_PACKER_DECL(border_left);
		STYLE_PACKER_DECL(border_right);
		STYLE_PACKER_DECL(border_top);
		STYLE_PACKER_DECL(column_rule);
		STYLE_PACKER_DECL(outline);
		STYLE_PACKER_DECL(colour);
		STYLE_PACKER_DECL(clip);
		STYLE_PACKER_DECL(flex_grow);
		STYLE_PACKER_DECL(flex_shrink);
		STYLE_PACKER_DECL(opacity);
		STYLE_PACKER_DECL(column_count);
		STYLE_PACKER_DECL(order);
		STYLE_PACKER_DECL(orphans);
		STYLE_PACKER_DECL(widows);
		STYLE_PACKER_DECL(z_index);
		STYLE_PACKER_DECL(column_gap);
		STYLE_PACKER_DECL(column_width);
		STYLE_PACKER_DECL(flex_basis);
		STYLE_PACKER_DECL(height);
		STYLE_PACKER_DECL(letter_spacing);
		STYLE_PACKER_DECL(margin_bottom);
		STYLE_PACKER_DECL(margin_left);
		STYLE_PACKER_DECL(margin_right);
		STYLE_PACKER_DECL(margin_top);
		STYLE_PACKER_DECL(max_height);
		STYLE_PACKER_DECL(max_width);
		STYLE_PACKER_DECL(min_height);
		STYLE_PACKER_DECL(min_width);
		STYLE_PACKER_DECL(padding_bottom);
		STYLE_PACKER_DECL(padding_left);
		STYLE_PACKER_DECL(padding_right);
		STYLE_PACKER_DECL(padding_top);
		STYLE_PACKER_DECL(text_indent);
		STYLE_PACKER_DECL(vertical_align);
		STYLE_PACKER_DECL(width);
		STYLE_PACKER_DECL(word_spacing);
		STYLE_PACKER_DECL(border_width_bottom);
		STYLE_PACKER_DECL(border_width_left);
		STYLE_PACKER_DECL(border_width_right);
		STYLE_PACKER_DECL(border_width_top);
		STYLE_PACKER_DECL(column_rule_width);
		STYLE_PACKER_DECL(font_size);
		STYLE_PACKER_DECL(line_height);
		STYLE_PACKER_DECL(outline_width);
		STYLE_PACKER_DECL(border_spacing);
		STYLE_PACKER_DECL(background_position);
		STYLE_PACKER_DECL(align_content);
		STYLE_PACKER_DECL(align_items);
		STYLE_PACKER_DECL(align_self);
		STYLE_PACKER_DECL(border_collapse);
		STYLE_PACKER_DECL(border_bottom_style);
		STYLE_PACKER_DECL(border_left_style);
		STYLE_PACKER_DECL(border_right_style);
		STYLE_PACKER_DECL(border_top_style);
		STYLE_PACKER_DECL(box_sizing);
		STYLE_PACKER_DECL(break_after);
		STYLE_PACKER_DECL(break_before);
		STYLE_PACKER_DECL(break_inside);
		STYLE_PACKER_DECL(caption_side);
		STYLE_PACKER_DECL(clear);
		STYLE_PACKER_DECL(column_fill);
		STYLE_PACKER_DECL(column_rule_style);
		STYLE_PACKER_DECL(column_span);
		STYLE_PACKER_DECL(direction);
		STYLE_PACKER_DECL(display_static);
		STYLE_PACKER_DECL(empty_cells);
		STYLE_PACKER_DECL(flex_direction);
		STYLE_PACKER_DECL(flex_wrap);
		STYLE_PACKER_DECL(float);
		STYLE_PACKER_DECL(font_style);
		STYLE_PACKER_DECL(font_variant);
		STYLE_PACKER_DECL(font_weight);
		STYLE_PACKER_DECL(justify_content);
		STYLE_PACKER_DECL(list_style_position);
		STYLE_PACKER_DECL(list_style_type);
		STYLE_PACKER_DECL(outline_style);
		STYLE_PACKER_DECL(overflow_x);
		STYLE_PACKER_DECL(overflow_y);
		STYLE_PACKER_DECL(page_break_after);
		STYLE_PACKER_DECL(page_break_before);
		STYLE_PACKER_DECL(page_break_inside);
		STYLE_PACKER_DECL(position);
		STYLE_PACKER_DECL(table_layout);
		STYLE_PACKER_DECL(text_align);
		STYLE_PACKER_DECL(text_decoration);
		STYLE_PACKER_DECL(text_transform);
		STYLE_PACKER_DECL(unicode_bidi);
		STYLE_PACKER_DECL(visibility);
		STYLE_PACKER_DECL(white_space);
		STYLE_PACKER_DECL(writing_mode);
		STYLE_PACKER_DECL(background_attachment);
		STYLE_PACKER_DECL(display);
		STYLE_PACKER_DECL(content);
		STYLE_PACKER_DECL(font_family);
	});
}

static int ext_declare_stylesheet(ExtensionState* s)
{
	int n_args = lua_gettop(s);
	luaL_argcheck(
		s, true, 1 <= n_args && n_args <= 2, "Expected either one or two arguments to " EM_IMPORT_STYLESHEET_FUNC_NAME);
	luaL_argcheck(s, true, lua_isstring(s, 1), "Expected string as first argument to " EM_IMPORT_STYLESHEET_FUNC_NAME);
	luaL_argcheck(s, true, lua_isstring(s, 2), "Expected string as second argument to " EM_IMPORT_STYLESHEET_FUNC_NAME);

	lua_getfield(s, LUA_REGISTRYINDEX, STYLESHEET_LIST_RIDX);

	// Get the new index
	lua_len(s, -1);
	int new_index = 1 + lua_tointeger(s, -1);
	lua_pop(s, 1);

	// Create the stylesheet info container
	lua_createtable(s, 2, 0);

	// Set the stylesheet info
	lua_pushnil(s);
	lua_copy(s, 1, -1);
	lua_pushnil(s);
	lua_copy(s, 2, -1);
	lua_seti(s, -3, 2);
	lua_seti(s, -2, 1);

	// Append the stylesheet info
	lua_seti(s, -2, new_index);

	lua_pop(s, 1);

	return 0;
}

int import_stylesheets_from_extensions(ExtensionState* s, Styler* styler, bool insert_css_into_context)
{
	int rc = 0;

	lua_getfield(s, LUA_REGISTRYINDEX, STYLESHEET_LIST_RIDX);
	lua_pushnil(s);
	while (lua_next(s, -2))
	{
		lua_geti(s, -1, 1);
		lua_geti(s, -2, 2);
		Str sheet_loc;
		make_strv(&sheet_loc, (char*)lua_tostring(s, -2));
		bool have_sheet_data;
		Str sheet_data;
		if ((have_sheet_data = lua_isstring(s, -1)))
			make_strv(&sheet_data, (char*)lua_tostring(s, -1));

		if (append_style_sheet(styler, &sheet_loc, have_sheet_data ? &sheet_data : NULL, insert_css_into_context))
		{
			log_err("Failed to import extension stylesheet '%s'", sheet_loc.str);
			rc = 1;
		}

		dest_str(&sheet_loc);
		if (have_sheet_data)
			dest_str(&sheet_data);

		lua_pop(s, 3);

		if (rc)
			break;
	}
	lua_pop(s, 1);
	return rc;
}

#define PACK(f, g)                                                                                                     \
	g;                                                                                                                 \
	lua_setfield(s, -2, #f);

#define PUSHER_DATA_DECLS                                                                                              \
	uint8_t ret;                                                                                                       \
	css_fixed length1;                                                                                                 \
	css_unit unit1;                                                                                                    \
	css_fixed length2;                                                                                                 \
	css_unit unit2;                                                                                                    \
	css_computed_clip_rect rect;                                                                                       \
	css_color colour;                                                                                                  \
	lwc_string** str_list;                                                                                             \
	int32_t integer;                                                                                                   \
	css_fixed fixed;                                                                                                   \
	const css_computed_content_item* content;                                                                          \
	bool root = node == node->parent;                                                                                  \
	int rc	  = 0;
#define PUSHER_DATA_DECLS_SUPPRESS_UNUSED                                                                              \
	UNUSED(ret);                                                                                                       \
	UNUSED(length1);                                                                                                   \
	UNUSED(unit1);                                                                                                     \
	UNUSED(length2);                                                                                                   \
	UNUSED(unit2);                                                                                                     \
	UNUSED(rect);                                                                                                      \
	UNUSED(colour);                                                                                                    \
	UNUSED(str_list);                                                                                                  \
	UNUSED(integer);                                                                                                   \
	UNUSED(fixed);                                                                                                     \
	UNUSED(content);                                                                                                   \
	UNUSED(root);

#define PUSH_SCALAR(f) lua_pushinteger(s, css_computed_##f(pestyle));

#define PUSH_SCALAR_VA(f, ...) lua_pushinteger(s, css_computed_##f(pestyle, __VA_ARGS__));

#define PUSH_INTEGER(f, set) PUSH_INTEGER2(f, ret == CSS_##set##_SET)
#define PUSH_INTEGER2(f, c)                                                                                            \
	ret = css_computed_##f(pestyle, &integer);                                                                         \
	if (c)                                                                                                             \
		lua_pushinteger(s, integer);                                                                                   \
	else                                                                                                               \
		lua_pushnil(s);

#define PUSH_INTEGER_FIXED(f, set) PUSH_INTEGER_FIXED2(f, ret == CSS_##set##_SET)
#define PUSH_INTEGER_FIXED2(f, c)                                                                                      \
	ret = css_computed_##f(pestyle, &fixed);                                                                           \
	if (c)                                                                                                             \
		lua_pushinteger(s, FIXTOFLT(fixed));                                                                           \
	else                                                                                                               \
		lua_pushnil(s);

#define PUSH_FIXED(f, set) PUSH_FIXED2(f, ret == CSS_##set##_SET)
#define PUSH_FIXED2(f, c)                                                                                              \
	ret = css_computed_##f(pestyle, &fixed);                                                                           \
	if (c)                                                                                                             \
		lua_pushnumber(s, FIXTOFLT(fixed));                                                                            \
	else                                                                                                               \
		lua_pushnil(s);

#define PUSH_LENGTH(f, set) PUSH_LENGTH2(f, ret == CSS_##set##_SET)
#define PUSH_LENGTH2(f, c)                                                                                             \
	ret = css_computed_##f(pestyle, &length1, &unit1);                                                                 \
	if (c)                                                                                                             \
	{                                                                                                                  \
		lua_createtable(s, 0, 2);                                                                                      \
		lua_pushnumber(s, FIXTOFLT(length1));                                                                          \
		lua_setfield(s, -2, "length");                                                                                 \
		lua_pushinteger(s, unit1);                                                                                     \
		lua_setfield(s, -2, "unit");                                                                                   \
	}                                                                                                                  \
	else                                                                                                               \
		lua_pushinteger(s, ret);

#define PUSH_INTEGER_OR_LENGTH(f, int_c, length_c)                                                                     \
	ret = css_computed_##f(pestyle, &length1, &unit1);                                                                 \
	if (int_c)                                                                                                         \
	{                                                                                                                  \
		lua_pushnumber(s, FIXTOFLT(length1));                                                                          \
	}                                                                                                                  \
	else if (length_c)                                                                                                 \
	{                                                                                                                  \
		lua_createtable(s, 0, 2);                                                                                      \
		lua_pushnumber(s, FIXTOFLT(length1));                                                                          \
		lua_setfield(s, -2, "length");                                                                                 \
		lua_pushinteger(s, unit1);                                                                                     \
		lua_setfield(s, -2, "unit");                                                                                   \
	}

#define PUSH_LENGTH_VH(f, set)                                                                                         \
	ret = css_computed_##f(pestyle, &length1, &unit1, &length2, &unit2);                                               \
	if (ret == CSS_##set##_SET)                                                                                        \
	{                                                                                                                  \
		lua_createtable(s, 0, 4);                                                                                      \
		lua_pushnumber(s, FIXTOFLT(length1));                                                                          \
		lua_setfield(s, -2, "hlength");                                                                                \
		lua_pushinteger(s, unit1);                                                                                     \
		lua_setfield(s, -2, "hunit");                                                                                  \
		lua_pushnumber(s, FIXTOFLT(length2));                                                                          \
		lua_setfield(s, -2, "vlength");                                                                                \
		lua_pushinteger(s, unit2);                                                                                     \
		lua_setfield(s, -2, "vunit");                                                                                  \
	}                                                                                                                  \
	else                                                                                                               \
		lua_pushinteger(s, ret);

#define GET_RGBA(colour, idx) ((colour >> (idx << 3)) & 0xff)

#define PUSH_COLOUR(f, set) PUSH_COLOUR2(f##_, ret == CSS_##set##_COLOR_COLOR)
#define PUSH_COLOUR2(f, c)                                                                                             \
	ret = css_computed_##f##color(pestyle, &colour);                                                                   \
	if (c)                                                                                                             \
	{                                                                                                                  \
		lua_createtable(s, 0, 4);                                                                                      \
		lua_pushinteger(s, GET_RGBA(colour, 3));                                                                       \
		lua_setfield(s, -2, "a");                                                                                      \
		lua_pushinteger(s, GET_RGBA(colour, 2));                                                                       \
		lua_setfield(s, -2, "r");                                                                                      \
		lua_pushinteger(s, GET_RGBA(colour, 1));                                                                       \
		lua_setfield(s, -2, "g");                                                                                      \
		lua_pushinteger(s, GET_RGBA(colour, 0));                                                                       \
		lua_setfield(s, -2, "b");                                                                                      \
	}                                                                                                                  \
	else                                                                                                               \
		lua_pushinteger(s, ret);

#define PUSH_COMPUTED_CLIP_RECT(f, c)                                                                                  \
	ret = css_computed_##f(pestyle, &rect);                                                                            \
	if (c)                                                                                                             \
	{                                                                                                                  \
		lua_createtable(s, 0, 12);                                                                                     \
		lua_pushnumber(s, FIXTOFLT(rect.top));                                                                         \
		lua_setfield(s, -2, "top");                                                                                    \
		lua_pushnumber(s, FIXTOFLT(rect.right));                                                                       \
		lua_setfield(s, -2, "right");                                                                                  \
		lua_pushnumber(s, FIXTOFLT(rect.bottom));                                                                      \
		lua_setfield(s, -2, "bottom");                                                                                 \
		lua_pushnumber(s, FIXTOFLT(rect.left));                                                                        \
		lua_setfield(s, -2, "left");                                                                                   \
		lua_pushinteger(s, rect.tunit);                                                                                \
		lua_setfield(s, -2, "tunit");                                                                                  \
		lua_pushinteger(s, rect.runit);                                                                                \
		lua_setfield(s, -2, "runit");                                                                                  \
		lua_pushinteger(s, rect.bunit);                                                                                \
		lua_setfield(s, -2, "bunit");                                                                                  \
		lua_pushinteger(s, rect.lunit);                                                                                \
		lua_setfield(s, -2, "lunit");                                                                                  \
		lua_pushboolean(s, rect.top_auto);                                                                             \
		lua_setfield(s, -2, "tauto");                                                                                  \
		lua_pushboolean(s, rect.right_auto);                                                                           \
		lua_setfield(s, -2, "rauto");                                                                                  \
		lua_pushboolean(s, rect.bottom_auto);                                                                          \
		lua_setfield(s, -2, "bauto");                                                                                  \
		lua_pushboolean(s, rect.left_auto);                                                                            \
		lua_setfield(s, -2, "lauto");                                                                                  \
	}                                                                                                                  \
	else                                                                                                               \
		lua_pushinteger(s, ret);

#define PUSH_STRING_LIST(f)                                                                                            \
	ret = css_computed_##f(pestyle, &str_list);                                                                        \
	lua_createtable(s, 0, 1);                                                                                          \
	lua_pushinteger(s, ret);                                                                                           \
	lua_setfield(s, -2, "type");                                                                                       \
	lua_newtable(s);                                                                                                   \
	if (str_list)                                                                                                      \
		for (int i = 1; *str_list; str_list++, i++)                                                                    \
		{                                                                                                              \
			lua_pushlstring(s, lwc_string_data(*str_list), lwc_string_length(*str_list));                              \
			lua_seti(s, -2, i);                                                                                        \
		}                                                                                                              \
	lua_setfield(s, -2, "list");

#define PUSH_CONTENT(f)                                                                                                \
	ret = css_computed_content(pestyle, &content);                                                                     \
	if (ret == CSS_CONTENT_SET)                                                                                        \
		rc |= pack_content(s, content, node);                                                                          \
	else                                                                                                               \
		lua_pushnil(s);

int pack_style(ExtensionState* s, Style* style, DocTreeNode* node)
{
	ASSERT(sizeof(pseudo_element_names) / sizeof(*pseudo_element_names) == CSS_PSEUDO_ELEMENT_COUNT);
	if (!style)
	{
		lua_pushnil(s);
		return 0;
	}

	PUSHER_DATA_DECLS;

	lua_createtable(s, 0, num_style_elements + CSS_PSEUDO_ELEMENT_COUNT - 1);
	for (int i = 0; i < CSS_PSEUDO_ELEMENT_COUNT; i++)
	{
		const css_computed_style* pestyle = style->styles[i];
		if (!pestyle)
			continue;
		if (i != CSS_PSEUDO_ELEMENT_NONE)
			lua_createtable(s, 0, num_style_elements);

		PACK(top, PUSH_LENGTH(top, TOP));
		PACK(bottom, PUSH_LENGTH(bottom, BOTTOM));
		PACK(left, PUSH_LENGTH(left, LEFT));
		PACK(right, PUSH_LENGTH(right, RIGHT));

		PACK(background, PUSH_COLOUR(background, BACKGROUND));
		PACK(border_bottom, PUSH_COLOUR(border_bottom, BORDER));
		PACK(border_left, PUSH_COLOUR(border_left, BORDER));
		PACK(border_right, PUSH_COLOUR(border_right, BORDER));
		PACK(border_top, PUSH_COLOUR(border_top, BORDER));
		PACK(column_rule, PUSH_COLOUR(column_rule, COLUMN_RULE));
		PACK(outline, PUSH_COLOUR(outline, OUTLINE));
		PACK(colour, PUSH_COLOUR2(, ret == CSS_COLOR_COLOR));

		PACK(clip, PUSH_COMPUTED_CLIP_RECT(clip, ret == CSS_CLIP_RECT));
		PACK(flex_grow, PUSH_FIXED(flex_grow, FLEX_GROW));
		PACK(flex_shrink, PUSH_FIXED(flex_shrink, FLEX_SHRINK));
		PACK(opacity, PUSH_FIXED(opacity, OPACITY));
		PACK(column_count, PUSH_INTEGER_FIXED(column_count, COLUMN_COUNT));
		PACK(order, PUSH_INTEGER(order, ORDER));
		PACK(orphans, PUSH_INTEGER(orphans, ORPHANS)); // Not working for some reason?
		PACK(widows, PUSH_INTEGER(widows, WIDOWS));	   // Not working for some reason?
		PACK(z_index, PUSH_INTEGER_FIXED(z_index, Z_INDEX));
		PACK(column_gap, PUSH_LENGTH(column_gap, COLUMN_GAP));
		PACK(column_width, PUSH_LENGTH(column_width, COLUMN_WIDTH));
		PACK(flex_basis, PUSH_LENGTH(flex_basis, FLEX_BASIS));
		PACK(height, PUSH_LENGTH(height, HEIGHT));
		PACK(letter_spacing, PUSH_LENGTH(letter_spacing, LETTER_SPACING));
		PACK(margin_bottom, PUSH_LENGTH(margin_bottom, MARGIN));
		PACK(margin_left, PUSH_LENGTH(margin_left, MARGIN));
		PACK(margin_right, PUSH_LENGTH(margin_right, MARGIN));
		PACK(margin_top, PUSH_LENGTH(margin_top, MARGIN));
		PACK(max_height, PUSH_LENGTH(max_height, MAX_HEIGHT));
		PACK(max_width, PUSH_LENGTH(max_width, MAX_WIDTH));
		PACK(min_height, PUSH_LENGTH(min_height, MIN_HEIGHT));
		PACK(min_width, PUSH_LENGTH(min_width, MIN_WIDTH));
		PACK(padding_bottom, PUSH_LENGTH(padding_bottom, PADDING));
		PACK(padding_left, PUSH_LENGTH(padding_left, PADDING));
		PACK(padding_right, PUSH_LENGTH(padding_right, PADDING));
		PACK(padding_top, PUSH_LENGTH(padding_top, PADDING));
		PACK(text_indent, PUSH_LENGTH(text_indent, TEXT_INDENT));
		PACK(vertical_align, PUSH_LENGTH(vertical_align, VERTICAL_ALIGN));
		PACK(width, PUSH_LENGTH(width, WIDTH));
		PACK(word_spacing, PUSH_LENGTH(word_spacing, WORD_SPACING));
		PACK(border_width_bottom, PUSH_LENGTH2(border_bottom_width, ret == CSS_BORDER_WIDTH_WIDTH));
		PACK(border_width_left, PUSH_LENGTH2(border_left_width, ret == CSS_BORDER_WIDTH_WIDTH));
		PACK(border_width_right, PUSH_LENGTH2(border_right_width, ret == CSS_BORDER_WIDTH_WIDTH));
		PACK(border_width_top, PUSH_LENGTH2(border_top_width, ret == CSS_BORDER_WIDTH_WIDTH));
		PACK(column_rule_width, PUSH_LENGTH2(column_rule_width, ret == CSS_COLUMN_RULE_WIDTH_WIDTH));
		PACK(font_size, PUSH_LENGTH2(font_size, ret == CSS_FONT_SIZE_DIMENSION));
		PACK(line_height,
			PUSH_INTEGER_OR_LENGTH(line_height, ret == CSS_LINE_HEIGHT_NUMBER, ret == CSS_LINE_HEIGHT_DIMENSION));
		PACK(outline_width, PUSH_LENGTH2(outline_width, ret == CSS_OUTLINE_WIDTH_WIDTH));
		if (!root) // v/h lengths don't initialise correctly for the root, seems to be a libcss bug?
		{
			PACK(border_spacing, PUSH_LENGTH_VH(border_spacing, BORDER_SPACING));
			PACK(background_position, PUSH_LENGTH_VH(background_position, BACKGROUND_POSITION));
		}

		PACK(align_content, PUSH_SCALAR(align_content));
		PACK(align_items, PUSH_SCALAR(align_items));
		PACK(align_self, PUSH_SCALAR(align_self));
		PACK(border_collapse, PUSH_SCALAR(border_collapse));
		PACK(border_bottom_style, PUSH_SCALAR(border_bottom_style));
		PACK(border_left_style, PUSH_SCALAR(border_left_style));
		PACK(border_right_style, PUSH_SCALAR(border_right_style));
		PACK(border_top_style, PUSH_SCALAR(border_top_style));
		PACK(box_sizing, PUSH_SCALAR(box_sizing));
		PACK(break_after, PUSH_SCALAR(break_after));
		PACK(break_before, PUSH_SCALAR(break_before));
		PACK(break_inside, PUSH_SCALAR(break_inside));
		PACK(caption_side, PUSH_SCALAR(caption_side));
		PACK(clear, PUSH_SCALAR(clear));
		PACK(column_fill, PUSH_SCALAR(column_fill));
		PACK(column_rule_style, PUSH_SCALAR(column_rule_style));
		PACK(column_span, PUSH_SCALAR(column_span));
		PACK(direction, PUSH_SCALAR(direction));
		PACK(display_static, PUSH_SCALAR(display_static));
		PACK(empty_cells, PUSH_SCALAR(empty_cells));
		PACK(flex_direction, PUSH_SCALAR(flex_direction));
		PACK(flex_wrap, PUSH_SCALAR(flex_wrap));
		PACK(float, PUSH_SCALAR(float));
		PACK(font_style, PUSH_SCALAR(font_style));
		PACK(font_variant, PUSH_SCALAR(font_variant));
		PACK(font_weight, PUSH_SCALAR(font_weight));
		PACK(justify_content, PUSH_SCALAR(justify_content));
		PACK(list_style_position, PUSH_SCALAR(list_style_position));
		PACK(list_style_type, PUSH_SCALAR(list_style_type));
		PACK(outline_style, PUSH_SCALAR(outline_style));
		PACK(overflow_x, PUSH_SCALAR(overflow_x));
		PACK(overflow_y, PUSH_SCALAR(overflow_y));
		PACK(page_break_after, PUSH_SCALAR(page_break_after));
		PACK(page_break_before, PUSH_SCALAR(page_break_before));
		PACK(page_break_inside, PUSH_SCALAR(page_break_inside));
		PACK(position, PUSH_SCALAR(position));
		PACK(table_layout, PUSH_SCALAR(table_layout));
		PACK(text_align, PUSH_SCALAR(text_align));
		PACK(text_decoration, PUSH_SCALAR(text_decoration));
		PACK(text_transform, PUSH_SCALAR(text_transform));
		PACK(unicode_bidi, PUSH_SCALAR(unicode_bidi));
		PACK(visibility, PUSH_SCALAR(visibility));
		PACK(white_space, PUSH_SCALAR(white_space));
		PACK(writing_mode, PUSH_SCALAR(writing_mode));
		PACK(background_attachment, PUSH_SCALAR(background_attachment));
		PACK(display, PUSH_SCALAR_VA(display, root));

		PACK(content, PUSH_CONTENT(content));

		PACK(font_family, PUSH_STRING_LIST(font_family));

		// -- url
		// background_image();
		// background_repeat();
		// list_style_image();
		// -- counter things
		// counter_increment();
		// counter_reset();
		// -- unsupported
		// cursor();
		// quotes();
		if (i != CSS_PSEUDO_ELEMENT_NONE)
			lua_setfield(s, -2, pseudo_element_names[i]);
	}
	return rc;
}

#define PUSH_CONTENT_STRING_AT(field)                                                                                  \
	str = lwc_string_ref(c->data.field);                                                                               \
	lua_pushlstring(s, lwc_string_data(str), lwc_string_length(str));                                                  \
	lwc_string_unref(str);

#define STYLE_PACKER(field, pusher)                                                                                    \
	STYLE_PACKER_SIG(field)                                                                                            \
	{                                                                                                                  \
		DocTreeNode* node = to_node(s, 1);                                                                             \
		Style* style	  = node->style;                                                                               \
		if (!style)                                                                                                    \
			return log_warn("no style"), 0;                                                                            \
		PseudoElemType ptype = lua_tointeger(s, 2);                                                                    \
		if (ptype < CSS_PSEUDO_ELEMENT_NONE || CSS_PSEUDO_ELEMENT_COUNT <= ptype)                                      \
			return luaL_error(s, "Invalid pseudo element index: %d", ptype);                                           \
		PseudoElemStyle* pestyle = style->styles[ptype];                                                               \
		if (!pestyle)                                                                                                  \
			return log_warn("no pe style"), 0;                                                                         \
		PUSHER_DATA_DECLS;                                                                                             \
		pusher;                                                                                                        \
		if (rc)\
			return luaL_error(s, "Failed to pack style '" #field "'");\
		return 1;                                                                                                      \
		PUSHER_DATA_DECLS_SUPPRESS_UNUSED;                                                                             \
	}

STYLE_PACKER(top, PUSH_LENGTH(top, TOP))
STYLE_PACKER(bottom, PUSH_LENGTH(bottom, BOTTOM))
STYLE_PACKER(left, PUSH_LENGTH(left, LEFT))
STYLE_PACKER(right, PUSH_LENGTH(right, RIGHT))
STYLE_PACKER(background, PUSH_COLOUR(background, BACKGROUND))
STYLE_PACKER(border_bottom, PUSH_COLOUR(border_bottom, BORDER))
STYLE_PACKER(border_left, PUSH_COLOUR(border_left, BORDER))
STYLE_PACKER(border_right, PUSH_COLOUR(border_right, BORDER))
STYLE_PACKER(border_top, PUSH_COLOUR(border_top, BORDER))
STYLE_PACKER(column_rule, PUSH_COLOUR(column_rule, COLUMN_RULE))
STYLE_PACKER(outline, PUSH_COLOUR(outline, OUTLINE))
STYLE_PACKER(colour, PUSH_COLOUR2(, ret == CSS_COLOR_COLOR))
STYLE_PACKER(clip, PUSH_COMPUTED_CLIP_RECT(clip, ret == CSS_CLIP_RECT))
STYLE_PACKER(flex_grow, PUSH_FIXED(flex_grow, FLEX_GROW))
STYLE_PACKER(flex_shrink, PUSH_FIXED(flex_shrink, FLEX_SHRINK))
STYLE_PACKER(opacity, PUSH_FIXED(opacity, OPACITY))
STYLE_PACKER(column_count, PUSH_INTEGER_FIXED(column_count, COLUMN_COUNT))
STYLE_PACKER(order, PUSH_INTEGER(order, ORDER))
STYLE_PACKER(orphans, PUSH_INTEGER(orphans, ORPHANS))
STYLE_PACKER(widows, PUSH_INTEGER(widows, WIDOWS))
STYLE_PACKER(z_index, PUSH_INTEGER_FIXED(z_index, Z_INDEX))
STYLE_PACKER(column_gap, PUSH_LENGTH(column_gap, COLUMN_GAP))
STYLE_PACKER(column_width, PUSH_LENGTH(column_width, COLUMN_WIDTH))
STYLE_PACKER(flex_basis, PUSH_LENGTH(flex_basis, FLEX_BASIS))
STYLE_PACKER(height, PUSH_LENGTH(height, HEIGHT))
STYLE_PACKER(letter_spacing, PUSH_LENGTH(letter_spacing, LETTER_SPACING))
STYLE_PACKER(margin_bottom, PUSH_LENGTH(margin_bottom, MARGIN))
STYLE_PACKER(margin_left, PUSH_LENGTH(margin_left, MARGIN))
STYLE_PACKER(margin_right, PUSH_LENGTH(margin_right, MARGIN))
STYLE_PACKER(margin_top, PUSH_LENGTH(margin_top, MARGIN))
STYLE_PACKER(max_height, PUSH_LENGTH(max_height, MAX_HEIGHT))
STYLE_PACKER(max_width, PUSH_LENGTH(max_width, MAX_WIDTH))
STYLE_PACKER(min_height, PUSH_LENGTH(min_height, MIN_HEIGHT))
STYLE_PACKER(min_width, PUSH_LENGTH(min_width, MIN_WIDTH))
STYLE_PACKER(padding_bottom, PUSH_LENGTH(padding_bottom, PADDING))
STYLE_PACKER(padding_left, PUSH_LENGTH(padding_left, PADDING))
STYLE_PACKER(padding_right, PUSH_LENGTH(padding_right, PADDING))
STYLE_PACKER(padding_top, PUSH_LENGTH(padding_top, PADDING))
STYLE_PACKER(text_indent, PUSH_LENGTH(text_indent, TEXT_INDENT))
STYLE_PACKER(vertical_align, PUSH_LENGTH(vertical_align, VERTICAL_ALIGN))
STYLE_PACKER(width, PUSH_LENGTH(width, WIDTH))
STYLE_PACKER(word_spacing, PUSH_LENGTH(word_spacing, WORD_SPACING))
STYLE_PACKER(border_width_bottom, PUSH_LENGTH2(border_bottom_width, ret == CSS_BORDER_WIDTH_WIDTH))
STYLE_PACKER(border_width_left, PUSH_LENGTH2(border_left_width, ret == CSS_BORDER_WIDTH_WIDTH))
STYLE_PACKER(border_width_right, PUSH_LENGTH2(border_right_width, ret == CSS_BORDER_WIDTH_WIDTH))
STYLE_PACKER(border_width_top, PUSH_LENGTH2(border_top_width, ret == CSS_BORDER_WIDTH_WIDTH))
STYLE_PACKER(column_rule_width, PUSH_LENGTH2(column_rule_width, ret == CSS_COLUMN_RULE_WIDTH_WIDTH))
STYLE_PACKER(font_size, PUSH_LENGTH2(font_size, ret == CSS_FONT_SIZE_DIMENSION))
STYLE_PACKER(
	line_height, PUSH_INTEGER_OR_LENGTH(line_height, ret == CSS_LINE_HEIGHT_NUMBER, ret == CSS_LINE_HEIGHT_DIMENSION))
STYLE_PACKER(outline_width, PUSH_LENGTH2(outline_width, ret == CSS_OUTLINE_WIDTH_WIDTH))
STYLE_PACKER(border_spacing, if (root) return 0; PUSH_LENGTH_VH(border_spacing, BORDER_SPACING))
STYLE_PACKER(background_position, if (root) return 0; PUSH_LENGTH_VH(background_position, BACKGROUND_POSITION))
STYLE_PACKER(align_content, PUSH_SCALAR(align_content))
STYLE_PACKER(align_items, PUSH_SCALAR(align_items))
STYLE_PACKER(align_self, PUSH_SCALAR(align_self))
STYLE_PACKER(border_collapse, PUSH_SCALAR(border_collapse))
STYLE_PACKER(border_bottom_style, PUSH_SCALAR(border_bottom_style))
STYLE_PACKER(border_left_style, PUSH_SCALAR(border_left_style))
STYLE_PACKER(border_right_style, PUSH_SCALAR(border_right_style))
STYLE_PACKER(border_top_style, PUSH_SCALAR(border_top_style))
STYLE_PACKER(box_sizing, PUSH_SCALAR(box_sizing))
STYLE_PACKER(break_after, PUSH_SCALAR(break_after))
STYLE_PACKER(break_before, PUSH_SCALAR(break_before))
STYLE_PACKER(break_inside, PUSH_SCALAR(break_inside))
STYLE_PACKER(caption_side, PUSH_SCALAR(caption_side))
STYLE_PACKER(clear, PUSH_SCALAR(clear))
STYLE_PACKER(column_fill, PUSH_SCALAR(column_fill))
STYLE_PACKER(column_rule_style, PUSH_SCALAR(column_rule_style))
STYLE_PACKER(column_span, PUSH_SCALAR(column_span))
STYLE_PACKER(direction, PUSH_SCALAR(direction))
STYLE_PACKER(display_static, PUSH_SCALAR(display_static))
STYLE_PACKER(empty_cells, PUSH_SCALAR(empty_cells))
STYLE_PACKER(flex_direction, PUSH_SCALAR(flex_direction))
STYLE_PACKER(flex_wrap, PUSH_SCALAR(flex_wrap))
STYLE_PACKER(float, PUSH_SCALAR(float))
STYLE_PACKER(font_style, PUSH_SCALAR(font_style))
STYLE_PACKER(font_variant, PUSH_SCALAR(font_variant))
STYLE_PACKER(font_weight, PUSH_SCALAR(font_weight))
STYLE_PACKER(justify_content, PUSH_SCALAR(justify_content))
STYLE_PACKER(list_style_position, PUSH_SCALAR(list_style_position))
STYLE_PACKER(list_style_type, PUSH_SCALAR(list_style_type))
STYLE_PACKER(outline_style, PUSH_SCALAR(outline_style))
STYLE_PACKER(overflow_x, PUSH_SCALAR(overflow_x))
STYLE_PACKER(overflow_y, PUSH_SCALAR(overflow_y))
STYLE_PACKER(page_break_after, PUSH_SCALAR(page_break_after))
STYLE_PACKER(page_break_before, PUSH_SCALAR(page_break_before))
STYLE_PACKER(page_break_inside, PUSH_SCALAR(page_break_inside))
STYLE_PACKER(position, PUSH_SCALAR(position))
STYLE_PACKER(table_layout, PUSH_SCALAR(table_layout))
STYLE_PACKER(text_align, PUSH_SCALAR(text_align))
STYLE_PACKER(text_decoration, PUSH_SCALAR(text_decoration))
STYLE_PACKER(text_transform, PUSH_SCALAR(text_transform))
STYLE_PACKER(unicode_bidi, PUSH_SCALAR(unicode_bidi))
STYLE_PACKER(visibility, PUSH_SCALAR(visibility))
STYLE_PACKER(white_space, PUSH_SCALAR(white_space))
STYLE_PACKER(writing_mode, PUSH_SCALAR(writing_mode))
STYLE_PACKER(background_attachment, PUSH_SCALAR(background_attachment))
STYLE_PACKER(display, PUSH_SCALAR_VA(display, root))
STYLE_PACKER(content, PUSH_CONTENT(content))
STYLE_PACKER(font_family, PUSH_STRING_LIST(font_family))

static int pack_content(ExtensionState* s, const css_computed_content_item* c, DocTreeNode* node)
{
	int rc = 0;
	lwc_string* str;
	lua_newtable(s);
	for (int i = 1; c->type; i++, c++)
	{
		switch (c->type)
		{
			case CSS_COMPUTED_CONTENT_NONE:
				lua_pushnil(s);
				break;
			case CSS_COMPUTED_CONTENT_STRING:
				PUSH_CONTENT_STRING_AT(string);
				break;
			case CSS_COMPUTED_CONTENT_URI:
				PUSH_CONTENT_STRING_AT(uri);
				break;
			case CSS_COMPUTED_CONTENT_COUNTER:
				rc = log_warn("Counters are not currently supported (called form CSS)");
				lua_pushliteral(s, "");
				break;
			case CSS_COMPUTED_CONTENT_COUNTERS:
				rc |= log_warn("Counters are not currently supported (called form CSS)");
				lua_pushliteral(s, "");
				break;
			case CSS_COMPUTED_CONTENT_ATTR:
				if (!strcmp("name", lwc_string_data(c->data.attr)))
					lua_pushlstring(s, node->name->str, node->name->len);
				else
				{
					rc |= log_warn("Unknown attribute in referenced CSS: attr(%s)", lwc_string_data(c->data.attr));
					lua_pushliteral(s, "");
				}
				break;
			case CSS_COMPUTED_CONTENT_OPEN_QUOTE:
				lua_pushliteral(s, "“"); // Would need to handle the quote depth
				break;
			case CSS_COMPUTED_CONTENT_CLOSE_QUOTE:
				lua_pushliteral(s, "”");
				break;
			case CSS_COMPUTED_CONTENT_NO_OPEN_QUOTE:
				lua_pushnil(s);
				break;
			case CSS_COMPUTED_CONTENT_NO_CLOSE_QUOTE:
				lua_pushnil(s);
				break;
			default:
				log_err("Unknown css content type: %d", c->type);
				lua_pushnil(s);
				rc = 1;
		}
		lua_seti(s, -2, i);
	}
	return rc;
}
