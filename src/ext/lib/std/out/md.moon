import driver_capabilities from require 'std.constants'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'

import TS_BASIC_STYLING from driver_capabilities

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
