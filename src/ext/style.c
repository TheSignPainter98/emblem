/**
 * @file style.c
 * @brief Implements function for loading stylesheets from extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "style.h"

#include "data/str.h"
#include "debug.h"
#include "logs/logs.h"
#include "lua-pointers.h"
#include "lua.h"
#include "pp/assert.h"
#include "style/css.h"
#include <lauxlib.h>
#include <libcss/fpmath.h>
#include <stdbool.h>
#include <string.h>

static const char* const pseudo_element_names[] = { "elem", "first_line", "first_letter", "before", "after" };

static int pack_content(ExtensionState* s, const css_computed_content_item* content, DocTreeNode* node);

void provide_styler(ExtensionEnv* e)
{
	lua_pushlightuserdata(e->state, e->styler);
	lua_setglobal(e->state, STYLER_LP_LOC);
}

int ext_import_stylesheet(ExtensionState* s)
{
	lua_getglobal(s, EM_ENV_VAR_NAME);
	ExtensionEnv* env;
	int rc = to_userdata_pointer((void**)&env, s, -1, EXT_ENV);
	if (rc)
		luaL_error(s, "Invalid environment pointer");
	lua_pop(s, 1);
	if (env->iter_num)
		luaL_error(s, "Stylesheets cannot be added after the `start` event has occurred");

	luaL_argcheck(s, true, lua_gettop(s) == 1, "Expected exactly one argument to " EM_IMPORT_STYLESHEET_FUNC_NAME);
	luaL_argcheck(s, true, lua_isstring(s, -1), "Expected string as argument to " EM_IMPORT_STYLESHEET_FUNC_NAME);
	Str sheet_loc;
	char* str = (char*)lua_tostring(s, -1);
	make_strv(&sheet_loc, str);

	lua_getglobal(s, STYLER_LP_LOC);
	Styler* styler;
	rc = to_userdata_pointer((void**)&styler, s, -1, STYLER);
	if (rc)
		luaL_error(s, "Invalid internal value");
	lua_pop(s, 1);

	log_debug("Got lua styler at %p", (void*)styler);

	if (append_style_sheet(styler, &sheet_loc))
		luaL_error(s, "Failed to import extension stylesheet '%s'", sheet_loc.str);
	return 0;
}

#define PACK_SCALAR(f)                                                                                                 \
	lua_pushinteger(s, css_computed_##f(pestyle));                                                                     \
	lua_setfield(s, -2, #f);

#define PACK_SCALAR_VA(f, ...)                                                                                         \
	lua_pushinteger(s, css_computed_##f(pestyle, __VA_ARGS__));                                                        \
	lua_setfield(s, -2, #f);

#define PACK_INTEGER(f, set) PACK_INTEGER2(f, ret == CSS_##set##_SET)
#define PACK_INTEGER2(f, c)                                                                                            \
	ret = css_computed_##f(pestyle, &integer);                                                                         \
	if (c)                                                                                                             \
	{                                                                                                                  \
		lua_pushinteger(s, integer);                                                                                   \
		lua_setfield(s, -2, #f);                                                                                       \
	}
#define PACK_INTEGER_FIXED(f, set) PACK_INTEGER_FIXED2(f, ret == CSS_##set##_SET)
#define PACK_INTEGER_FIXED2(f, c)                                                                                      \
	ret = css_computed_##f(pestyle, &fixed);                                                                           \
	if (c)                                                                                                             \
	{                                                                                                                  \
		lua_pushinteger(s, FIXTOFLT(fixed));                                                                           \
		lua_setfield(s, -2, #f);                                                                                       \
	}

#define PACK_FIXED(f, set) PACK_FIXED2(f, ret == CSS_##set##_SET)
#define PACK_FIXED2(f, c)                                                                                              \
	ret = css_computed_##f(pestyle, &fixed);                                                                           \
	if (c)                                                                                                             \
	{                                                                                                                  \
		lua_pushnumber(s, FIXTOFLT(fixed));                                                                            \
		lua_setfield(s, -2, #f);                                                                                       \
	}

#define PACK_LENGTH(f, set) PACK_LENGTH2(f, ret == CSS_##set##_SET)
#define PACK_LENGTH2(f, c)                                                                                             \
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
	{                                                                                                                  \
		lua_pushinteger(s, ret);                                                                                       \
	}                                                                                                                  \
	lua_setfield(s, -2, #f);
#define PACK_INTEGER_OR_LENGTH(f, int_c, length_c)                                                                     \
	ret = css_computed_##f(pestyle, &length1, &unit1);                                                                 \
	if (int_c)                                                                                                         \
	{                                                                                                                  \
		lua_pushnumber(s, FIXTOFLT(length1));                                                                          \
		lua_setfield(s, -2, #f);                                                                                       \
	}                                                                                                                  \
	else if (length_c)                                                                                                 \
	{                                                                                                                  \
		lua_createtable(s, 0, 2);                                                                                      \
		lua_pushnumber(s, FIXTOFLT(length1));                                                                          \
		lua_setfield(s, -2, "length");                                                                                 \
		lua_pushinteger(s, unit1);                                                                                     \
		lua_setfield(s, -2, "unit");                                                                                   \
		lua_setfield(s, -2, #f);                                                                                       \
	}

#define PACK_LENGTH_VH(f, set)                                                                                         \
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
	{                                                                                                                  \
		lua_pushinteger(s, ret);                                                                                       \
	}                                                                                                                  \
	lua_setfield(s, -2, #f);

#define GET_RGBA(colour, idx) ((colour >> (idx << 3)) & 0xff)

#define PACK_COLOUR(f, set) PACK_COLOUR2(f##_, ret == CSS_##set##_COLOR_COLOR)
#define PACK_COLOUR2(f, c)                                                                                             \
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
	{                                                                                                                  \
		lua_pushinteger(s, ret);                                                                                       \
	}                                                                                                                  \
	lua_setfield(s, -2, #f "colour");

#define PACK_COMPUTED_CLIP_RECT(f, c)                                                                                  \
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
	{                                                                                                                  \
		lua_pushinteger(s, ret);                                                                                       \
	}                                                                                                                  \
	lua_setfield(s, -2, #f);

#define PACK_STRING_LIST(f)                                                                                            \
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
	lua_setfield(s, -2, "list");                                                                                       \
	lua_setfield(s, -2, #f);

int pack_style(ExtensionState* s, Style* style, DocTreeNode* node)
{
	ASSERT(sizeof(pseudo_element_names) / sizeof(*pseudo_element_names) == CSS_PSEUDO_ELEMENT_COUNT);
	if (!style)
	{
		lua_pushnil(s);
		return 0;
	}

	uint8_t ret;
	css_fixed length1;
	css_unit unit1;
	css_fixed length2;
	css_unit unit2;
	css_computed_clip_rect rect;
	css_color colour;
	lwc_string** str_list;
	int32_t integer;
	css_fixed fixed;
	bool root = !node->parent;
	int rc	  = 0;

	lua_createtable(s, 0, CSS_PSEUDO_ELEMENT_COUNT);
	for (int i = 0; i < CSS_PSEUDO_ELEMENT_COUNT; i++)
	{
		const css_computed_style* pestyle = style->styles[i];
		if (!pestyle)
			continue;
		const int num_style_elements = 59;
		lua_createtable(s, 0, num_style_elements);

		PACK_LENGTH(top, TOP);
		PACK_LENGTH(bottom, BOTTOM);
		PACK_LENGTH(left, LEFT);
		PACK_LENGTH(right, RIGHT);

		PACK_COLOUR(background, BACKGROUND);
		PACK_COLOUR(border_bottom, BORDER);
		PACK_COLOUR(border_left, BORDER);
		PACK_COLOUR(border_right, BORDER);
		PACK_COLOUR(border_top, BORDER);
		PACK_COLOUR(column_rule, COLUMN_RULE);
		PACK_COLOUR(outline, OUTLINE);
		PACK_COLOUR2(, ret == CSS_COLOR_COLOR);

		PACK_COMPUTED_CLIP_RECT(clip, ret == CSS_CLIP_RECT);
		PACK_FIXED(flex_grow, FLEX_GROW);
		PACK_FIXED(flex_shrink, FLEX_SHRINK);
		PACK_FIXED(opacity, OPACITY);
		PACK_INTEGER_FIXED(column_count, COLUMN_COUNT);
		PACK_INTEGER(order, ORDER);
		PACK_INTEGER(orphans, ORPHANS); // Not working for some reason?
		PACK_INTEGER(widows, WIDOWS);	// Not working for some reason?
		PACK_INTEGER_FIXED(z_index, Z_INDEX);
		PACK_LENGTH(column_gap, COLUMN_GAP);
		PACK_LENGTH(column_width, COLUMN_WIDTH);
		PACK_LENGTH(flex_basis, FLEX_BASIS);
		PACK_LENGTH(height, HEIGHT);
		PACK_LENGTH(letter_spacing, LETTER_SPACING);
		PACK_LENGTH(margin_bottom, MARGIN);
		PACK_LENGTH(margin_left, MARGIN);
		PACK_LENGTH(margin_right, MARGIN);
		PACK_LENGTH(margin_top, MARGIN);
		PACK_LENGTH(max_height, MAX_HEIGHT);
		PACK_LENGTH(max_width, MAX_WIDTH);
		PACK_LENGTH(min_height, MIN_HEIGHT);
		PACK_LENGTH(min_width, MIN_WIDTH);
		PACK_LENGTH(padding_bottom, PADDING);
		PACK_LENGTH(padding_left, PADDING);
		PACK_LENGTH(padding_right, PADDING);
		PACK_LENGTH(padding_top, PADDING);
		PACK_LENGTH(text_indent, TEXT_INDENT);
		PACK_LENGTH(vertical_align, VERTICAL_ALIGN);
		PACK_LENGTH(width, WIDTH);
		PACK_LENGTH(word_spacing, WORD_SPACING);
		PACK_LENGTH2(border_bottom_width, ret == CSS_BORDER_WIDTH_WIDTH);
		PACK_LENGTH2(border_left_width, ret == CSS_BORDER_WIDTH_WIDTH);
		PACK_LENGTH2(border_right_width, ret == CSS_BORDER_WIDTH_WIDTH);
		PACK_LENGTH2(border_top_width, ret == CSS_BORDER_WIDTH_WIDTH);
		PACK_LENGTH2(column_rule_width, ret == CSS_COLUMN_RULE_WIDTH_WIDTH);
		PACK_LENGTH2(font_size, ret == CSS_FONT_SIZE_DIMENSION);
		PACK_INTEGER_OR_LENGTH(line_height, ret == CSS_LINE_HEIGHT_NUMBER, ret == CSS_LINE_HEIGHT_DIMENSION)
		PACK_LENGTH2(outline_width, ret == CSS_OUTLINE_WIDTH_WIDTH);
		if (!root) // v/h lengths don't initialise correctly for the root, seems to be a libcss bug?
		{
			PACK_LENGTH_VH(border_spacing, BORDER_SPACING);
			PACK_LENGTH_VH(background_position, BACKGROUND_POSITION);
		}

		PACK_SCALAR(align_content);
		PACK_SCALAR(align_items);
		PACK_SCALAR(align_self);
		PACK_SCALAR(border_collapse);
		PACK_SCALAR(border_bottom_style);
		PACK_SCALAR(border_left_style);
		PACK_SCALAR(border_right_style);
		PACK_SCALAR(border_top_style);
		PACK_SCALAR(box_sizing);
		PACK_SCALAR(break_after);
		PACK_SCALAR(break_before);
		PACK_SCALAR(break_inside);
		PACK_SCALAR(caption_side);
		PACK_SCALAR(clear);
		PACK_SCALAR(column_fill);
		PACK_SCALAR(column_rule_style);
		PACK_SCALAR(column_span);
		PACK_SCALAR(direction);
		PACK_SCALAR(display_static);
		PACK_SCALAR(empty_cells);
		PACK_SCALAR(flex_direction);
		PACK_SCALAR(flex_wrap);
		PACK_SCALAR(float);
		PACK_SCALAR(font_style);
		PACK_SCALAR(font_variant);
		PACK_SCALAR(font_weight);
		PACK_SCALAR(justify_content);
		PACK_SCALAR(list_style_position);
		PACK_SCALAR(list_style_type);
		PACK_SCALAR(outline_style);
		PACK_SCALAR(overflow_x);
		PACK_SCALAR(overflow_y);
		PACK_SCALAR(page_break_after);
		PACK_SCALAR(page_break_before);
		PACK_SCALAR(page_break_inside);
		PACK_SCALAR(position);
		PACK_SCALAR(table_layout);
		PACK_SCALAR(text_align);
		PACK_SCALAR(text_decoration);
		PACK_SCALAR(text_transform);
		PACK_SCALAR(unicode_bidi);
		PACK_SCALAR(visibility);
		PACK_SCALAR(white_space);
		PACK_SCALAR(writing_mode);
		PACK_SCALAR(background_attachment);
		PACK_SCALAR_VA(display, root);

		const css_computed_content_item* content;
		ret = css_computed_content(pestyle, &content);
		if (ret == CSS_CONTENT_SET)
		{
			rc |= pack_content(s, content, node);
			lua_setfield(s, -2, "content");
		}

		PACK_STRING_LIST(font_family);

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
		lua_setfield(s, -2, pseudo_element_names[i]);
	}
	return rc;
}

#define PACK_CONTENT_STRING_AT(field)                                                                                  \
	str = lwc_string_ref(c->data.field);                                                                               \
	lua_pushlstring(s, lwc_string_data(str), lwc_string_length(str));                                                  \
	lwc_string_unref(str);

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
				PACK_CONTENT_STRING_AT(string);
				break;
			case CSS_COMPUTED_CONTENT_URI:
				PACK_CONTENT_STRING_AT(uri);
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
