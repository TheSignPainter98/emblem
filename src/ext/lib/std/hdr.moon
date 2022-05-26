---
-- @file std.hdr
-- @brief Provides headings and tables of contents
-- @author Edward Jones
-- @date 2021-09-17

import Call, Content, Word from require 'std.ast'
import Directive, em, eval_string, stylesheet from require 'std.base'
import Counter, SyncList from require 'std.events'
import set_label from require 'std.ref'
import extend from require 'std.util'
import concat, insert from table
import rep from string

stylesheet 'std/hdr.scss'

class ContentsElement
	new: (@title, @ref, @lvl) =>
	show: => Call 'toc-item', { Call "toc-item-l#{@lvl}", @title }

---
-- @brief Represents a table of contents
class Toc extends SyncList
	new: (@toc_name='Table of Contents', @contents_max_depth=3) => super!
	output: =>
		(Call 'h1*', @toc_name) .. { content\show! for content in *@contents when content.lvl <= @contents_max_depth } .. Word 'asdf'


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
		heading = c\eval_string!
		toc\add ContentsElement heading, ref, i
		"#{ref}. #{heading}"

{ :Toc, :toc }
