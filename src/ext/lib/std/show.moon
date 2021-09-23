---
-- @file std.show
-- @brief Provides functions to show the values of lua-tables in more human-readible formats
-- @author Edward Jones
-- @date 2021-09-17

import is_list from require 'std.util'
import rep from string
import concat from table

show = (v) ->
	switch type v
		when 'boolean', 'nil', 'number', 'thread'
			tostring(v)
		when 'function', 'userdata'
			"(#{tostring(v)})"
		when 'string'
			"'#{v}'"
		when 'table'
			if is_list v
				return '[' .. (concat [ show e for e in *v ], ',') .. ']'
			'{' .. (concat [ (show e) .. ':' .. show val for e,val in pairs v ], ',') .. '}'
			else
				print 'Unknown type', type v

showp = (v) ->
	next = next
	_showp = (v, i) ->
		switch type v
			when 'string'
				v
			when 'table'
				pref = rep '  ', i
				if is_list v
					itm_pref = '\n' .. pref .. '- '
					return itm_pref .. (concat [ _showp e, i + 1 for e in *v ], itm_pref)
				if (next v) == nil
					return '{}'
				else
					return concat [ (tostring k) .. ': ' .. (_showp val, i + 1) for k,val in pairs v ], '\n' .. pref
				else
					show v
	_showp v, 0

import Directive, em, vars from require 'std.base'
em.vars = Directive 0, 0, "show variable scopes", -> show vars
{ :show, :showp }
