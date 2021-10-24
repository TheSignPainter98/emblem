---
-- @file std.out.drivers
-- @brief Provides a framework for handling and creating output drivers
-- @author Edward Jones
-- @date 2021-09-24

import css, driver_capabilities, node_types from require 'std.constants'
import unknown_x_msg from require 'std.edit'
import key_list from require 'std.func'
import em, SanitisedKeyTable from require 'std.base'
import log_err, log_warn from require 'std.log'
import sorted from require 'std.util'
import concat from table

local open
if not io.module_unavailable
	import open from io

import DISPLAY_BLOCK from css.display

curr_output_driver = nil

---
-- @brief Holds a a mapping of known languages to their associated drivers
output_drivers = SanitisedKeyTable!

---
-- @brief Represents an output driver
class OutputDriver
	new: (@support=0, @output_extension, @requires_stylesheet=false) =>
	format: (doc) => error "Output driver does not implement the 'format' class method"
	output: (doc, use_stdout, @stem, @generation_time) =>
		if not open
			log_warn "Extension-space output drivers unavailable due to sandbox level"
			return
		fname = stem .. '.' .. @output_extension
		output = @format doc

		if use_stdout
			print output
		elseif output
			f = open fname, 'w'
			log_err "Failed to open file #{fname}" unless f
			with f
				\write output
				\close!

import WORD, CALL, CONTENT from node_types

---
-- @brief Represents an output driver for a context-free language
class ContextFreeOutputDriver extends OutputDriver
	new: (@do_wrap_root=false, ...) => super ...
	special_tag_map: {}
	general_tag_enclose: (t, r, t2=t) => error "Context free output driver does not implement the 'general_tag_enclose' class method"
	special_tag_enclose: (t, r, t2=t) => error "Context free output driver does not implement the 'special_tag_enclose' class method"
	style_responses: {}
	par_inner_sep: ' '
	wrap_root: (r) => r
	first_block: true
	sanitise: (w) => w
	enclose_tag: (name, fmtd) =>
		special_tag = @special_tag_map[name]
		tag = special_tag or name
		tag_encloser = special_tag and @special_tag_enclose or @general_tag_enclose
		tag_encloser @, tag, fmtd
	format: (doc) =>
		format = (n, can_space) ->
			return '' unless n
			switch n.type
				when WORD
					@sanitise n.word
				when CALL
					special_tag = @special_tag_map[n.name]
					tag = special_tag or n.name
					result = format n.result, false
					return '' if result == ''

					initial = ''
					if can_space
						initial = ' '
					local tag_encloser
					if special_tag
						tag_encloser = @special_tag_enclose
					else
						tag_encloser = @general_tag_enclose
					tag_encloser @, tag, result
				when CONTENT
					concat [ format n.content[i], 1 < i for i=1,#n.content ], ' '
				else
					error "Unknown node type #{n.type}"
		ret = format doc, false
		ret = @wrap_root ret if @do_wrap_root
		ret

---
-- @brief Represents an output driver for context-free markup languages where paragraphs are represented by whitespace
class TextualMarkupOutputDriver extends ContextFreeOutputDriver
	general_tag_enclose: (t, r) => r
	next_delimiter: ''
	is_block: (n) => n.style.elem.display == DISPLAY_BLOCK
	format: (doc) =>
		@have_output = false
		format = (n, do_delimit) ->
			return '' unless n
			delimiter = do_delimit and @next_delimiter or ''
			@prev_delimiter = @next_delimiter unless delimiter == ''
			@next_delimiter = ' '

			local post_delimiter
			if @is_block n
				delimiter = '\n\n' if @have_output and @prev_delimiter != '\n\n'
				post_delimiter = '\n\n'
			else
				post_delimiter = ' '

			switch n.type
				when WORD
					@have_output = true
					delimiter ..  @sanitise n.word
				when CALL
					result = format n.result, false
					if result == ''
						result
					else
						r = delimiter .. (@style n, @enclose_tag n.name, result)
						@next_delimiter = post_delimiter
						r
				when CONTENT
					delimiter .. (concat [ format n.content[i], 1 < i for i=1,#n.content ])
				else
					error "Unknown node type #{n.type}"
		ret = format doc, false, false
		ret = @wrap_root ret if @do_wrap_root
		ret
	style: (node, fmtd) =>
		elem_style = node.style.elem
		for style in *@style_responses
			local open, close
			open, close = style elem_style
			if open or close
				fmtd = @special_tag_enclose open, fmtd, close
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
