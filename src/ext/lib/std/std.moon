import concat, insert, sort, unpack from table
import rep from string
import open from io
import load from require 'lyaml'

export em = {}

export id = (x) -> x
export do_nothing = (x) ->

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

export show = (v) ->
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

export showp = (v) ->
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

export is_list = (l) ->
	type = type
	if (type l) != 'table'
		return false
	maxk = -1
	for k,_ in pairs l
		if (type k) != 'number'
			return false
		maxk = k if maxk < k
	return maxk == #l

export keys = (t) ->
	[ k for k,_ in pairs t ]
export values = (t) ->
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

node_string = (n) ->
	switch n.type
		when 0
			return n.word
		when 1
			return node_string n.result
		when 2,4
			return concat [ node_string w for w in *n.line ], ' '
		when 3,5
			return concat [ node_string w for w in *n.lines ], '\n'
		else
			error "Unrecognised node type '#{n.type}'"
			return 1

eval_string = (d) -> node_string eval d

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
	em["h#{i}"] = ->
		text = "Hello this is an h#{i} heading?"
		ref = concat (extend [ c.val for c in *heading_counters[,i - 1] ], { heading_counters[i]\use! }), '.'
		ret = ref .. " " .. text
		toc\add {ret, i}
		ret

elem = (v, vs) ->
	for _,v2 in pairs vs
		if v == v2
			return true
	false

export sorted = (t, ...) ->
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

-- export cite_str = (key) ->
	-- switch cite_style
		-- when 'numeric'
			-- "[123]"
		-- else
			-- 'ERROR'

-- export bib_str = (key) ->
	-- switch cite_style
		-- when 'numeric'
			-- bib[key].author .. '.' .. bib[key].title .. ' in ' .. bib[key].date.year
		-- else
			-- 'ERROR'

-- em.cite = (key) ->
	-- used_bib[key] = BibItem(key) if not used_bib[key]
	-- used_bib[key]\cite!

stylers = { 'it', 'bf', 'sc' }
for styler in *stylers
	_G[styler] = (node) ->
		{
			type: 3
			name: styler
			args: { node }
		}

-- stylesheet 'share/toc.scss'

lvl = 0
em.test_func = (...) ->
	lvl += 1

	-- things = {}
	-- x = nil
	-- for k,v in pairs{...}
		-- print (rep '\t', lvl) .. k,v
		-- insert things, eval v
		-- print (rep '\t', lvl) .. '==='
		-- x = v
		-- -- print ast_type_name (things[k])
		-- -- print '___'
	-- -- print eval "ahfjkl"

	-- -- print ast_type_name 0
	-- -- for i in *{ 0, 1, 2, 3, 4, 5, 6 }
		-- -- print i
	-- -- print show {...}
	-- print 'things:', show things
	-- print x
	lvl -= 1
	return false
	-- return x

{:Component, :Toc, :em}
