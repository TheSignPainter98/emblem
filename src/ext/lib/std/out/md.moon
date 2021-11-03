---
-- @file std.out.md
-- @brief Provides an output driver for [markdown](https://www.wikiwand.com/en/Markdown)
-- @author Edward Jones
-- @date 2021-09-24

import css, driver_capabilities from require 'std.constants'
import StyleResponse, TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'

import TS_CSS_STYLES from driver_capabilities
import DISPLAY_BLOCK from css.display
import FONT_FAMILY_MONOSPACE from css.font_family
import FONT_STYLE_ITALIC, FONT_STYLE_OBLIQUE from css.font_style
import FONT_WEIGHT_BOLD, FONT_WEIGHT_BOLDER, FONT_WEIGHT_700, FONT_WEIGHT_800, FONT_WEIGHT_900 from css.font_weight
import TEXT_DECORATION_LINE_THROUGH from css.text_decoration

---
-- @brief Represents an output driver for markdown
class MarkdownOutputDriver extends TextualMarkupOutputDriver
	new: => super false, TS_CSS_STYLES, 'md'
	special_tag_enclose: (t, r, t2=t) =>
		if t\match '#+'
			{ t, ' ', r }
		else
			{ t, r, t2 }
	special_tag_map:
		h1: '#'
		h2: '##'
		h3: '###'
		h4: '####'
		h5: '#####'
		h6: '######'
		'h1*': '#'
		'h2*': '##'
		'h3*': '###'
		'h4*': '####'
		'h5*': '#####'
		'h6*': '######'
	style_responses: {
		=> '_' if @font_style == FONT_STYLE_ITALIC or @font_style == FONT_STYLE_OBLIQUE
		=>
			fw = @font_weight
			'**' if fw == FONT_WEIGHT_BOLD or
				fw == FONT_WEIGHT_BOLDER or
				fw == FONT_WEIGHT_700 or
				fw == FONT_WEIGHT_800 or
				fw == FONT_WEIGHT_900
		=> '~~' if @text_decoration == TEXT_DECORATION_LINE_THROUGH
		=> if @font_family.type == FONT_FAMILY_MONOSPACE
			if @display != DISPLAY_BLOCK
				'`' else '```\n', '\n```'
	}
	sanitise: (w) =>
		w = w\gsub '([*\\])', '\\%1'
		w = w\gsub '^_', '\\_'
		w = w\gsub '_$', '\\_'
		w

output_drivers.md = MarkdownOutputDriver!

{ :MarkdownOutputDriver }
