import driver_capabilities, node_types from require 'std.constants'
import key_list from require 'std.func'
import em, SanitisedKeyTable from require 'std.base'
import log_err, log_warn from require 'std.log'
import sorted from require 'std.util'
import concat from table

local open
if not io.module_unavailable
	import open from io

curr_output_driver = nil

output_drivers = SanitisedKeyTable!

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

class ContextFreeOutputDriver extends OutputDriver
	new: (@do_wrap_root=false, ...) => super ...
	special_tag_map: {}
	general_tag_enclose: (t, r) => error "Context free output driver does not implement the 'general_tag_enclose' class method"
	special_tag_enclose: (t, r) => error "Context free output driver does not implement the 'special_tag_enclose' class method"
	par_inner_sep: ' '
	wrap_root: (r) => r
	first_block: true
	format: (doc) =>
		format = (n, can_space, par_content) ->
			return '' unless n
			switch n.type
				when WORD
					local result
					if can_space
						result = ' ' .. n.word
					else
						result = n.word
					return result
				when CALL
					special_tag = @special_tag_map[n.name]
					tag = special_tag or n.name
					result = format n.result, false, n.name == 'p'
					return result if result == ''

					initial = ""
					if can_space
						initial = " "
					local tag_encloser
					if special_tag
						tag_encloser = @special_tag_enclose
					else
						tag_encloser = @general_tag_enclose
					initial .. tag_encloser @, tag, result

				when CONTENT
					intertext = ''
					intertext = @par_inner_sep if par_content
					concat [ format n.content[i], i > 1, false for i=1,#n.content ], intertext
				else
					error "Unknown node type #{n.type}"
		ret = format doc, false, false
		ret = @wrap_root ret if @do_wrap_root
		ret

class TextualMarkupOutputDriver extends ContextFreeOutputDriver
	general_tag_enclose: (t, r) =>
		if t == 'p'
			if @first_block
				@first_block = false
				r
			else
				@par_enclose r
		else
			@general_non_par_tag_enclose t, r
	general_non_par_tag_enclose: (_, r) => r
	par_enclose: (r) => '\n\n' .. r

get_output_driver = -> curr_output_driver

set_output_driver = (dname) ->
	if curr_output_driver != nil
		log_err "The output driver cannot be set more than once"
	unless curr_output_driver = output_drivers[dname]
		log_err "Unknown output driver '#{dname}', known drivers:" .. concat [ "\n\t#{d}" for d in *sorted key_list output_drivers ]

{ :get_output_driver, :set_output_driver, :ContextFreeOutputDriver, :TextualMarkupOutputDriver, :output_drivers }
