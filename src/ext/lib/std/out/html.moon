---
-- @file std.out.html
-- @brief Provides an output driver for [HTML](https://developer.mozilla.org/en-US/docs/Web/HTML)
-- @author Edward Jones
-- @date 2021-09-24

import driver_capabilities from require 'std.constants'
import ContextFreeOutputDriver, output_drivers from require 'std.out.drivers'
import concat from table

import TS_BASIC_STYLING, TS_COLOUR, TS_IMAGE, TS_TEXT_SIZE, TS_SVG from driver_capabilities

---
-- @brief Represents an output driver for bbcode
class HtmlOutputDriver extends ContextFreeOutputDriver
	new: (do_wrap_root) =>
		support = TS_BASIC_STYLING | TS_COLOUR | TS_IMAGE | TS_TEXT_SIZE | TS_SVG
		super do_wrap_root, support, 'html', true
	wrap_root: (r) =>
		concat {
			"<!DOCTYPE html>",
			"<!-- This file was generated by `em` on #{@generation_time}. -->",
			"<!-- Any changes will be overwritten next time typesetting is run -->",
			"<html>",
			"	<head>",
			"		<link rel=\"stylesheet\" type=\"text/css\" href=\"#{@stem}.css\"/>",
			"		<title>#{@stem\gsub '^%./', ''}</title>",
			"	</head>",
			"	<body>",
			r
			"	</body>",
			"</html>",
		}, '\n'
	special_tag_enclose: (t, r) => "<#{t} class=\"#{t}\">#{r}</#{t}>"
	general_tag_enclose: (t, r) => "<span class=\"#{t}\">#{r}</span>"
	par_enclose: (...) => @special_tag_enclose ...
	special_tag_map:
		p: "p"
		h1: "h1"
		h2: "h2"
		h3: "h3"
		h4: "h4"
		h5: "h5"
		h6: "h6"
		'h1*': "h1"
		'h2*': "h2"
		'h3*': "h3"
		'h4*': "h4"
		'h5*': "h5"
		'h6*': "h6"
	sanitise: (w) =>
		w = w\gsub '<', '&lt;'
		w = w\gsub '>', '&gt;'
		w

output_drivers.html = HtmlOutputDriver true
output_drivers.html_bare = HtmlOutputDriver false

{ :HtmlOutputDriver }
