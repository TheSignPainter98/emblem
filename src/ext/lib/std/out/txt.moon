---
-- @file txt.moon
-- @brief Provides an output driver for raw text
-- @author Edward Jones
-- @date 2021-10-25

import em from require 'std.base'
import driver_capabilities, node_flags, node_types from require 'std.constants'
import output_drivers, OutputDriver from require 'std.out.drivers'
import StringBuilder from require 'std.util'

import TS_NONE from driver_capabilities
import GLUE_LEFT, NBSP_LEFT from node_flags
import CALL, CONTENT, WORD from node_types

class RawOutputDriver extends OutputDriver
	new: => super TS_NONE, 'txt'
	format: (doc) =>
		sb = StringBuilder!
		delimiter = nil
		format = (n) ->
			return unless n
			delimiter = nil if delimiter and (n.flags & GLUE_LEFT) != 0
			switch n.type
				when WORD
					sb .. delimiter if delimiter
					sb .. (n.pword or n.word)
					delimiter = ' '
				when CALL
					format n.result
				when CONTENT
					format c for c in *n.content
				else
					error "Unknown node type #{n.type}"
		format doc
		sb!

output_drivers.txt = RawOutputDriver!
