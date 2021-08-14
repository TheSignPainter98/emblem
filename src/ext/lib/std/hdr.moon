import em, eval_string from require 'std.base'
import Counter, SyncList from require 'std.events'
import set_label from require 'std.ref'
import extend from require 'std.util'
import concat, insert from table
import rep from string

-- stylesheet 'share/hdr.scss'

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


toc = Toc!
em.toc = toc\output

heading_counters = {}
for i = 1,6
	insert heading_counters, Counter!
	if i > 1
		heading_counters[i - 1]\add_sub_counter heading_counters[i]
	em["h#{i}"] = (c) ->
		ref = concat (extend [ c.val for c in *heading_counters[,i - 1] ], { heading_counters[i]\use! }), '.'
		set_label ref
		ret = ref .. " " .. eval_string c
		toc\add {ret, i}
		ret

{ :Toc }
