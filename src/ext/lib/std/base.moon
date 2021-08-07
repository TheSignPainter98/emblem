import concat, insert, sort, unpack from table
import rep from string
import open from io
import load from require 'lyaml'

id = (x) -> x
do_nothing = (x) ->

components = {}
class Component extends {}
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
	_showp = (v, i) ->
		switch type v
			when 'string'
				v
			when 'table'
				pref = rep '  ', i
				if is_list v
					itm_pref = '\n' .. pref .. '- '
					return itm_pref .. (concat [ _showp e, i + 1 for e in *v ], itm_pref)
				next = next
				if (next v) == nil
					return '{}'
				else
					return concat [ (tostring k) .. ': ' .. (_showp val, i + 1) for k,val in pairs v ], '\n' .. pref
				else
					show v
	_showp v, 0

is_list = (l) ->
	type = type
	if (type l) != 'table'
		return false
	maxk = -1
	for k,_ in pairs l
		if (type k) != 'number'
			return false
		maxk = k if maxk < k
	return maxk == #l

keys = (t) ->
	[ k for k,_ in pairs t ]
values = (t) ->
	[ v for _,v in pairs t ]



-- class ExampleComponent extends Component
-- new: =>
-- super!
-- print 'Created an ExampleComponent'
-- on_start: =>
-- print 'Hello start!'
-- on_end: =>
-- print 'Hi, end!'
-- on_iter_start: =>
-- print 'Component iter start'
-- on_iter_end: =>
-- print 'Component iter end'

-- ex = ExampleComponent!

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

extend = (a1, a2) ->
	a3 = {unpack a1}
	for v in *a2
		insert a3, v
	return a3

eq = (a,b) ->
	ta = type a
	tb = type b
	if ta != tb
		return false

	if ta != 'table' and tb != 'table'
		return a == b

	mt = getmetatable a
	if mt and mt.__eq
		return a == b

	for k1,v1 in pairs a
		v2 = b[k1]
		if v2 == nil or not eq v1,v2
			return false

	for k2,v2 in pairs b
		v1 = a[k2]
		if v1 == nil or not eq v1,v2
			return false
	true

filter = (as, f=(a)->a!=nil) ->
	bs = {}
	for k,v in pairs as
		bs[k] = v if f v
	bs

filter_list = (as, f=(a)->a!=nil) ->
	bs = {}
	for v in *as
		insert bs, v if f v
	bs

node_string = (n) ->
	if n == nil
		return nil
	if 'table' != type n
		return tostring n
	switch n.type
		when node_types.word
			return n.word
		when node_types.call
			return node_string n.result
		when node_types.content
			return concat (filter_list [ node_string w for w in *n.content ]), ' '
		else
			error "Unrecognised node type '#{n.type}'"
			return 1

eval_string = (d) ->
	if 'userdata' == type d
		return node_string eval d
	tostring d

class SyncContainer extends Component
	new: =>
		super!
		@contents = {}
		@new_contents = {}
	on_iter_start: =>
		super!
		@contents = @new_contents
		@new_contents = {}
	on_iter_end: =>
		super!
		if not eq @contents, @new_contents
			requires_reiter!
	add: =>
		error "Function not implemented"
	output: =>
		error "Function not implemented"

class SyncList extends SyncContainer
	add: (c) =>
		insert @new_contents, c

class SyncSet extends SyncContainer
	add: (c) =>
		@new_contents[c] = true

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
		ret = ref .. " " .. eval_string c
		toc\add {ret, i}
		ret

elem = (v, vs) ->
	for _,v2 in pairs vs
		if v == v2
			return true
	false

sorted = (t, ...) ->
	sort t, ...
	t

class Bib extends SyncSet
	new: =>
		super!
		@has_src = false
		@bib = {}
		@unknown_citations = {}
	on_iter_start: =>
		super!
		@unknown_citations = {}
	on_end: =>
		super!
		if not eq @unknown_citations, {}
			print "The following citations were not known:\n\t" .. concat (sorted keys @unknown_citations), '\n\t'
	add: (c) =>
		super c
		if not @bib[c]
			@unknown_citations[c] = true
	read: (srcd) =>
		if not @has_src
			src = eval_string srcd

			f = open (eval_string srcd), 'r'
			if not f
				error "Failed to open file #{src}"
			lines = f\read '*all'
			f\close!

			@bib = load lines
			@has_src = true
	output: =>
		included_bib = {}
		for ref, itm in pairs @bib
			if @contents[ref]
				insert included_bib, itm

		sort included_bib, (i1, i2) ->
			fields = { 'author', 'title', 'year' }
			for field in *fields[,#fields - 1]
				if i1[field] != i2[field]
					return i1[field] < i2[field]
			return i1[fields[#fields]] < i2[fields[#fields]]

		'BIBLIOGRAPHY GOES HERE! ' .. show included_bib

bib = Bib!
em.bib = (src) ->
	bib\read src
	bib\output!
em.cite = (ref) ->
	bib\add eval_string ref


-- -- bibs = {}
-- -- class Bib
	-- -- new: (src) =>
		-- -- @bib = {}
		-- -- insert bibs, @

-- used_bib = {}
-- bib =
	-- chistikov_hitting_2019:
		-- author: 'Chistikov'
		-- title: 'Hitting families of size k'
		-- date:
			-- year: 2017
	-- asdf_fda_123:
		-- author: 'fdas'
		-- title: 'fdsafdafdfdsa'
		-- date:
			-- year: 2021

-- class BibItem extends Component
	-- new: (@ref) =>
		-- super 'BibItem'
		-- @curr_val = nil
		-- @bib_entry = nil
	-- cite: =>
		-- if @curr_val then @curr_val else '??'
	-- on_iter_end: (n) =>
		-- super!
		-- if n == 1
			-- @curr_val = cite_str @ref if bib[@ref]
			-- @bib_entry = bib_str @ref if bib[@ref]

-- em.bib = ->
	-- formatted_bib = {}
	-- for item in *values used_bib
		-- insert formatted_bib, item.bib_entry
	-- sorted formatted_bib, ((a,b) -> false)
	-- return formatted_bib

-- local cite_style
	-- cite_style = 'numeric'

-- cite_str = (key) ->
	-- switch cite_style
		-- when 'numeric'
			-- "[123]"
		-- else
			-- 'ERROR'

-- bib_str = (key) ->
	-- switch cite_style
		-- when 'numeric'
			-- bib[key].author .. '.' .. bib[key].title .. ' in ' .. bib[key].date.year
		-- else
			-- 'ERROR'

-- em.cite = (key) ->
	-- used_bib[key] = BibItem(key) if not used_bib[key]
	-- used_bib[key]\cite!

{
	word: word_type,
	content: content_type,
	call: call_type
} = node_types
class Node
	new: (@type) =>
	__tostring: => show @

sanitise_concat_input = (x) ->
	return {} if x == nil
	return {x} if ('table' != type x) or x.type == word_type or x.type == call_type
	return x.content if x.type == content_type
	error "Unrecognised concatenation input: #{show x}"

local Content
concat_ast_nodes = (as, bs) ->
	as = sanitise_concat_input as
	bs = sanitise_concat_input bs
	newlist = [ a for a in *as ]
	insert newlist, b for b in *bs
	Content newlist

class Word extends Node
	new: (@word) => super word_type
	__concat: concat_ast_nodes

class Content extends Node
	new: (@content) => super content_type
	__concat: concat_ast_nodes

class Call extends Node
	new: (@name, args) =>
		super call_type
		if is_list args
			@args = args
		else
			@args = {args}
	__concat: concat_ast_nodes
	__shl: (c, a) ->
		if 'table' != type c or c.type != call_type
			error "Left operand to an argument-append must be a call, instead got #{show c}"
		newargs = [ arg for arg in *c.args ]
		insert newargs, a
		Call c.name, newargs

mkcall = (name) -> (args) -> Call name, args

styles = { 'it', 'bf', 'sc', 'af', 'tt' }
stylers = { s, mkcall s for s in *styles }
import bf from stylers

-- stylesheet 'share/toc.scss'

{ :Component, :SyncContainer, :SyncSet, :SyncList, :Toc, :eval, :node_string, :eval_string, :id, :do_nothing, :show, :showp, :keys, :values, :sorted, :mkcall, :is_list}
