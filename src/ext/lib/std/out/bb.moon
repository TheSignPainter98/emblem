---
-- @file std.out.bb
-- @brief Provides an output driver for [BBCode](https://www.bbcode.org)
-- @author Edward Jones
-- @date 2021-09-24

import floor from math
import css, driver_capabilities from require 'std.constants'
import StyleResponse, TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'
import colour_to_hex from require 'std.style'

import TS_CSS_STYLES from driver_capabilities
import unit_str from css
import DISPLAY_BLOCK from css.display
import FONT_SIZE_XX_SMALL, FONT_SIZE_X_SMALL, FONT_SIZE_SMALL, FONT_SIZE_MEDIUM, FONT_SIZE_LARGE, FONT_SIZE_X_LARGE, FONT_SIZE_XX_LARGE, FONT_SIZE_SMALLER, FONT_SIZE_LARGER from css.font_size
import FONT_FAMILY_MONOSPACE from css.font_family
import FONT_STYLE_ITALIC, FONT_STYLE_OBLIQUE from css.font_style
import FONT_WEIGHT_BOLD, FONT_WEIGHT_BOLDER, FONT_WEIGHT_700, FONT_WEIGHT_800, FONT_WEIGHT_900 from css.font_weight
import TEXT_ALIGN_CENTRE, TEXT_ALIGN_JUSTIFY, TEXT_ALIGN_LEFT, TEXT_ALIGN_RIGHT from css.text_align
import TEXT_DECORATION_LINE_THROUGH, TEXT_DECORATION_UNDERLINE from css.text_decoration

font_size_strings =
	[FONT_SIZE_XX_SMALL]: 'xx-small'
	[FONT_SIZE_X_SMALL]: 'x-small'
	[FONT_SIZE_SMALL]: 'small'
	[FONT_SIZE_MEDIUM]: 'medium'
	[FONT_SIZE_LARGE]: 'large'
	[FONT_SIZE_X_LARGE]: 'x-large'
	[FONT_SIZE_XX_LARGE]: 'xx-large'
	[FONT_SIZE_SMALLER]: 'smaller'
	[FONT_SIZE_LARGER]: 'larger'

---
-- @brief Represents an output driver for bbcode
class BBCodeOutputDriver extends TextualMarkupOutputDriver
	new: (...) => super false, TS_CSS_STYLES, 'bb', ...
	special_tag_enclose: (t, r, t2=t) => { '[', t, ']', r, '[/', t2, ']' }
	special_tag_map:
		pre: 'pre'
		quote: 'quote'
		ul: 'list type=decimal'
		ol: 'list'
		li: 'li'
		img: 'img'
		sub: 'sub'
		sup: 'sup'
		url: 'url'
		more: 'more'
		spoiler: 'spoiler'
		hr: 'hr'
	style_responses: {
		=> if @font_family.type == FONT_FAMILY_MONOSPACE
			if @display != DISPLAY_BLOCK
				'tt' else 'code'
		=> 'i' if @font_style == FONT_STYLE_ITALIC or @font_style == FONT_STYLE_OBLIQUE
		=>
			fw = @font_weight
			'b' if fw == FONT_WEIGHT_BOLD or
				fw == FONT_WEIGHT_BOLDER or
				fw == FONT_WEIGHT_700 or
				fw == FONT_WEIGHT_800 or
				fw == FONT_WEIGHT_900
		=> 'u' if @text_decoration == TEXT_DECORATION_UNDERLINE
		=> 's' if @text_decoration == TEXT_DECORATION_LINE_THROUGH
		=> "color=##{colour_to_hex @colour}", 'color' if 'table' == type @colour
		=> 'center' if @text_align == TEXT_ALIGN_CENTRE
		=> 'left' if @text_align == TEXT_ALIGN_LEFT or @text_align == TEXT_ALIGN_JUSTIFY
		=> 'right' if @text_align == TEXT_ALIGN_RIGHT
		=> "font=#{@font_family.list[1]}", 'font' if 1 <= #@font_family.list and @font_family.type != FONT_FAMILY_MONOSPACE
		=>
			local size_str
			if 'number' == type @font_size
				size_str = font_size_strings[@font_size]
			else if 'table' == type @font_size
				size_str = "#{floor @font_size.length}#{unit_str[@font_size.unit]}"
			"size=#{size_str}", 'size' if size_str
	}

output_drivers.bb = BBCodeOutputDriver!

{ :BBCodeOutputDriver }
