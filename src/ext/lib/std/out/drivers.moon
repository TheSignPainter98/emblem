---
-- @file std.out.drivers
-- @brief Provides a framework for handling and creating output drivers
-- @author Edward Jones
-- @date 2021-09-24

import css, driver_capabilities, node_flags, node_types from require 'std.constants'
import unknown_x_msg from require 'std.edit'
import key_list from require 'std.func'
import SanitisedKeyTable from require 'std.base'
import log_err, log_warn from require 'std.log'
import sorted, StringBuilder from require 'std.util'
import concat from table
import __em from _G

local open
if not io.module_unavailable
	import open from io

import DISPLAY_BLOCK from css.display
import TS_NONE from driver_capabilities
import GLUE_LEFT, NBSP_LEFT from node_flags
import WORD, CALL, CONTENT from node_types

curr_output_driver = nil

---
-- @brief Holds a a mapping of known languages to their associated drivers
output_drivers = SanitisedKeyTable!

---
-- @brief Represents an output driver
class OutputDriver
	new: (@support=TS_NONE, @output_extension) =>
	format: (doc) => error "Output driver does not implement the 'format' class method"
	output: (doc, use_stdout, @stem, @generation_time) =>
		if not open
			log_warn "Extension-space output drivers unavailable due to sandbox level"
			return
		fname = stem .. '.' .. @output_extension
		doc = __em.nodes[doc]
		output = @format doc

		if use_stdout
			print output
		elseif output
			f = open fname, 'w'
			log_err "Failed to open file #{fname}" unless f
			with f
				\write output
				\close!

---
-- @brief Represents an output driver for a context-free language
class ContextFreeOutputDriver extends OutputDriver
	new: (@do_wrap_root=false, @nbsp='', ...) => super ...
	special_tag_map: {}
	general_tag_enclose: (t, r, t2=t) => error "Context free output driver does not implement the 'general_tag_enclose' class method"
	special_tag_enclose: (t, r, t2=t, as) => error "Context free output driver does not implement the 'special_tag_enclose' class method"
	style_responses: {}
	par_inner_sep: ' '
	wrap_root: (r) => r
	first_block: true
	sanitise: (w) => w
	enclose_tag: (name, fmtd, as, close=name) =>
		special_tag = @special_tag_map[name]
		tag = special_tag or name
		tag_encloser = special_tag and @\special_tag_enclose or @\general_tag_enclose
		tag_encloser tag, fmtd, close, as
	format: (doc) =>
		format = (n) ->
			return '' unless n
			switch n.type
				when WORD
					@sanitise n.pretty
				when CALL
					result = format n.result, false
					return '' if result == ''
					@enclose_tag n.name, result, n.args
				when CONTENT
					ret = {}
					for i = 1, 2 * #n - 1
						m = n[1 + i // 2]
						if i % 2 == 1
							ret[i] = format m
						else if (m.flags & GLUE_LEFT) != 0
							ret[i] = ''
						else if (m.flags & NBSP_LEFT) != 0
							ret[i] = @nbsp
						else
							ret[i] = ' '
					ret
				else
					error "Unknown node type #{n.type}"
		ret = StringBuilder format doc, false
		ret = @wrap_root ret if @do_wrap_root
		ret!

---
-- @brief Represents an output driver for context-free markup languages where paragraphs are represented by whitespace
class TextualMarkupOutputDriver extends ContextFreeOutputDriver
	general_tag_enclose: (t, r) => r
	next_delimiter: ''
	is_block: (n) => n.style and n.style.display == DISPLAY_BLOCK
	enclose_tag: (t, r, as, t2=t) => super t, r, t2, as
	format: (doc) =>
		@have_output = false
		format = (n, do_delimit) ->
			return '' unless n
			delimiter = ''
			if do_delimit and @have_output
				if (n.flags & NBSP_LEFT) != 0
					delimiter = @nbsp
				else if (n.flags & GLUE_LEFT) == 0
					delimiter = @next_delimiter
			@next_delimiter = ' '

			local post_delimiter
			if @is_block n
				delimiter = '\n\n' if @have_output and @prev_delimiter != '\n\n'
				post_delimiter = '\n\n'
			else
				post_delimiter = ' '

			@prev_delimiter = delimiter unless delimiter == ''

			switch n.type
				when WORD
					@have_output = true
					{ delimiter, @sanitise n.pretty }
				when CALL
					result = format n.result, false
					if result == ''
						result
					else
						r = { delimiter, @style n, @enclose_tag n.name, result, n.args }
						@next_delimiter = post_delimiter
						r
				when CONTENT
					{ delimiter, [ format n[i], 1 < i for i=1,#n ] }
				else
					error "Unknown node type #{n.type}"
		ret = StringBuilder format doc, false, false
		ret = @wrap_root ret if @do_wrap_root
		ret!
	style: (node, fmtd, as) =>
		return fmtd unless node.style
		elem_style = node.style
		for style in *@style_responses
			local open, close
			open, close = style elem_style
			if open or close
				fmtd = @special_tag_enclose open, fmtd, close, as
		fmtd

---
-- @brief Gets the current output driver
-- @return The current output driver
get_output_driver = -> curr_output_driver

---
-- @brief Sets the current output driver
-- @param dname The name of the new output driver
set_output_driver = (dname) ->
	if curr_output_driver != nil
		log_err "The output driver cannot be set more than once"
	unless curr_output_driver = output_drivers[dname]
		log_err unknown_x_msg 'output driver', dname, key_list output_drivers

{ :get_output_driver, :set_output_driver, :OutputDriver, :ContextFreeOutputDriver, :TextualMarkupOutputDriver, :output_drivers }
