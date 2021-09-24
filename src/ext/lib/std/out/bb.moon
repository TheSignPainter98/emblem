---
-- @file std.out.bb
-- @brief Provides an output driver for [BBCode](https://www.bbcode.org)
-- @author Edward Jones
-- @date 2021-09-24

import driver_capabilities from require 'std.constants'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'

import TS_BASIC_STYLING, TS_COLOUR, TS_TEXT_SIZE from driver_capabilities

---
-- @brief Represents an output driver for bbcode
class BBCodeOutputDriver extends TextualMarkupOutputDriver
	support: TS_BASIC_STYLING | TS_COLOUR | TS_TEXT_SIZE
	output_extension: 'bb'
	special_tag_enclose: (t, r) => "[#{t}]#{r}[/#{t}]"
	special_tag_map:
		bf: 'b'
		it: 'i'
		sc: 'u'
		af: 's'
		pre: 'pre'
		quote: 'quote'
		tt: 'tt'
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
		justify: 'justify'
		left: 'left'
		centre: 'center'
		center: 'center'
		right: 'right'

output_drivers.bb = BBCodeOutputDriver!

{ :BBCodeOutputDriver }
