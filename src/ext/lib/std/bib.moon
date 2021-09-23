---
-- @file std.bib
-- @brief Provides bibliographies and citations
-- @author Edward Jones
-- @date 2021-09-17

import Call, Content, Word from require 'std.ast'
import copy_loc, Directive, em, eval_string, iter_num from require 'std.base'
import SyncBox, SyncSet from require 'std.events'
import map, value_list from require 'std.func'
import log_warn_here, log_warn_at_loc from require 'std.log'
import stylers from require 'std.style'
import eq, sorted from require 'std.util'
import concat, insert, sort from table
import load from require 'lyaml'

import it from stylers

-- stylesheet 'share/bib.scss'

cite_style = 'numeric'
set_cite_style = (style) -> cite_style = style
get_cite_style = -> cite_style
cite_styles =
	numeric: (itm) -> itm.bib_idx
get_cite_str = (...) -> cite_styles[cite_style](...)

unknown_citation_str = '??'

class BibItem
	new: (@ref, tbl) =>
		for field in *{ 'author', 'title', 'year' }
			@[field] = tbl[field]
		@cite_str = SyncBox unknown_citation_str

	cite: => "[#{@cite_str\value!}]"
	set_bib_idx: (@bib_idx) => @cite_str\set get_cite_str @

	__lt: (i1, i2) ->
		fields = { 'author', 'title', 'year' }
		for field in *fields[,#fields - 1]
			if i1[field] != i2[field]
				return i1[field] < i2[field]
		i1[fields[#fields]] < i2[fields[#fields]]

class UnknownCitation
	new: (@ref) =>
		@loc = copy_loc!
	__lt: (u, v) -> u.ref < v.ref
	__tostring: => @ref

class Bib extends SyncSet
	new: (@bib_name='Bibliography') =>
		super!
		@bib = {}
		@unknown_citations = {}
	on_iter_start: =>
		super!
		@unknown_citations = {}
	on_end: =>
		super!
		if not eq @unknown_citations, {}
			log_warn_at_loc u.loc, "Non-existant citation '#{u.ref}'" for u in *sorted @unknown_citations
	add: (c) =>
		c = eval_string c
		super c
		if not @bib[c]
			insert @unknown_citations, UnknownCitation c
			"[#{unknown_citation_str}]"
		else
			@bib[c]\cite!
	get_file: (fname) =>
		f = open fname, 'r'
		if not f
			return false, {}
		lines = f\read '*all'
		f\close!
		true, lines
	load_bib: (bib) => @bib[k] = BibItem k, v for k,v in pairs bib
	read: (raw_src='bib') =>
		src = eval_string raw_src
		local acceptible_srcs
		if src\match '%.'
			acceptible_srcs = {src}
		else
			acceptible_srcs = { src, src .. '.yml', src .. '.yaml', src .. '.json' }
		succ = false
		local lines
		for src in *acceptible_srcs
			succ, lines = @get_file src
			break if succ
		if succ
			@load_bib load lines
		else
			attempts = ''
			if #acceptible_srcs > 1
				attempts = ", tried: #{concat [ "'#{f}'" for f in *acceptible_srcs ], ', '}"
			log_warn_here "Could not find bibliography file '#{src}'#{attempts}"
	output: =>
		included_bib = sorted [ itm for ref,itm in pairs @bib when @contents[ref] ]
		itm\set_bib_idx i for i,itm in pairs included_bib

		bib_table = Content [ (Word itm\cite!) .. itm.author .. (it itm.title) .. itm.year for itm in *included_bib ]
		(Call 'h1*', @bib_name) .. bib_table

bib = Bib!
em.bib = Directive 1, 0, "Create the main bibliography using the given source file", (src) ->
	if iter_num! == 1
		bib\read src
	bib\output!
em.cite = Directive 1, 0, "Cite a given reference", (ref) -> bib\add ref

{ :Bib, :get_cite_style, :set_cite_style, :cite_styles }
