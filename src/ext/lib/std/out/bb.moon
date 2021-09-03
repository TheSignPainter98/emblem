import driver_capabilities from require 'std.constants'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'

import TS_BASIC_STYLING, TS_COLOUR, TS_TEXT_SIZE from driver_capabilities

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
