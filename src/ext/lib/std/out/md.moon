---
-- @file std.out.md
-- @brief Provides an output driver for [markdown](https://www.wikiwand.com/en/Markdown)
-- @author Edward Jones
-- @date 2021-09-24

import driver_capabilities from require 'std.constants'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'

import TS_BASIC_STYLING from driver_capabilities

---
-- @brief Represents an output driver for markdown
class MarkdownOutputDriver extends TextualMarkupOutputDriver
	new: =>
		support = TS_BASIC_STYLING
		super false, support, 'md'
	special_tag_enclose: (t, r) =>
		if t\match '#+'
			@first_block = false
			t .. ' ' .. r
		else
			t .. r .. t
	special_tag_map:
		bf: '**'
		it: '_'
		tt: '`'
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

output_drivers.md = MarkdownOutputDriver!

{ :MarkdownOutputDriver }
