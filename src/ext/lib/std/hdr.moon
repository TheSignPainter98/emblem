---
-- @file std.hdr
-- @brief Provides headings and tables of contents
-- @author Edward Jones
-- @date 2021-09-17

import Directive, em, eval_string from require 'std.base'
import Counter, SyncList from require 'std.events'
import set_label from require 'std.ref'
import extend from require 'std.util'
import concat, insert from table
import rep from string

-- stylesheet 'share/hdr.scss'

---
-- @brief Represents a table of contents
class Toc extends SyncList
	new: =>
		super!
		@contents_max_depth = 3
	output: =>
		-- 'Table of contents ' .. show @contents
		formatted_contents = {}
		for contents_line in *@contents
			if contents_line[2] <= @contents_max_depth
				insert formatted_contents, (rep '&nbsp;', contents_line[2]) .. contents_line[1]
		concat formatted_contents, '</br>'

---
-- @brief The main table of contents
toc = Toc!
em.toc = Directive 0, 0, "Create a table of contents", toc\output

heading_counters = {}
for i = 1,6
	insert heading_counters, Counter!
	if i > 1
		heading_counters[i - 1]\add_sub_counter heading_counters[i]
	em["h#{i}"] = Directive 1, 0, "Create a level #{i} header", (c) ->
		ref = concat (extend [ c.val for c in *heading_counters[,i - 1] ], { heading_counters[i]\use! }), '.'
		set_label ref
		ret = ref .. " " .. eval_string c
		toc\add {ret, i}
		ret

{ :Toc, :toc }
