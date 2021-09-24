---
-- @file std.events
-- @brief Prpvides functionality for responding to typesetting events
-- @author Edward Jones
-- @date 2021-09-17

import requires_reiter from require 'std.base'
import show, ShowTable from require 'std.show'
import do_nothing, filter_list from require 'std.func'
import elem, eq, non_nil from require 'std.util'
import concat, insert from table

components = {}
---
-- @brief Represents a component of a document which can respond to typesetting events
class Component
	new: => insert components, @
	on_start: do_nothing
	on_iter_start: do_nothing
	on_iter_end: do_nothing
	on_end: do_nothing

events = {
	'on_start'
	'on_iter_start'
	'on_iter_end'
	'on_end'
}
for event in *events
	_G[event] = (...) ->
		for comp in *components
			comp[event](comp, ...) if comp[event] != do_nothing

---
-- @brief Represents a counter component. Resets at the start of each typesetting pass. Can have sub-counters which also cause it to reset.
class Counter extends Component
	new: =>
		super!
		@sub_counters = {}
		@val = 0
	use: =>
		@inc!
		@val
	inc: =>
		@val += 1
		@reset_subs!
	reset: =>
		@val = 0
		@reset_subs!
	reset_subs: =>
		for c in *@sub_counters
			c\reset!
	add_sub_counter: (c) => insert @sub_counters, c

	on_start: =>
		super!
		@reset!
	on_iter_start: =>
		super!
		@reset!

---
-- @brief Represents a value-container which requests a typesetting loop re-run if its value at the end of the current run differs from that at the end of the last
class SyncContainer extends Component
	new: (@initial={}) =>
		super!
		@contents = @initial
		@new_contents = @initial
	on_iter_start: =>
		super!
		@contents = @new_contents
		@new_contents = @initial
	on_iter_end: =>
		super!
		if not eq @contents, @new_contents
			requires_reiter!
	add: =>
		error "Function not implemented"

---
-- @brief Represnts a sync container of a single primitive value
class SyncBox extends SyncContainer
	new: (@initial=0) => super @initial
	set: (v) => @new_contents = v
	value: => @contents

---
-- @brief Represents a sync container of a list of elements
class SyncList extends SyncContainer
	add: (c) => insert @new_contents, c
	has: (c) => elem c, @new_contents

---
-- @brief Represents a sync container of a set of values
class SyncSet extends SyncContainer
	add: (c) => @new_contents[c] = true
	has: (c) => @new_contents[c]

---
-- @brief Represents a subc container which represents a map
class SyncMap extends SyncContainer
	add: (k,v) => @new_contents[k] = v
	has: (k) => @contents[k] != nil
	get: (k) => @contents[k]

{ :Component, :SyncContainer, :SyncBox, :SyncList, :SyncSet, :SyncMap, :Counter }
