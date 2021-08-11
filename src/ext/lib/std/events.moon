import requires_reiter from require 'std.base'
import show, ShowTable from require 'std.show'
import do_nothing, filter_list from require 'std.func'
import eq, non_nil from require 'std.util'
import concat, insert from table

components = {}
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
	output: =>
		error "Function not implemented"

class SyncBox extends SyncContainer
	new: (@initial=0) => super @initial
	set: (v) => @new_contents = v
	value: => @contents

class SyncList extends SyncContainer
	add: (c) =>
		insert @new_contents, c

class SyncSet extends SyncContainer
	add: (c) =>
		@new_contents[c] = true

{ :Component, :SyncContainer, :SyncBox, :SyncList, :SyncSet, :Counter }
