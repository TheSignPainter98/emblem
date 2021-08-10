import open from io
import Call, Content, Word from require 'std.ast'
import em, eq, eval_string from require 'std.base'
import SyncBox, SyncSet from require 'std.events'
import co_to_table, key_list, map from require 'std.func'
import log_warn from require 'std.log'
import stylers from require 'std.style'
import eq, sorted from require 'std.util'
import concat, sort from table
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
			log_warn "The following citations were not known:\n\t" .. concat (sorted key_list @unknown_citations), '\n\t'
	add: (c) =>
		c = eval_string c
		super c
		if not @bib[c]
			@unknown_citations[c] = true
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
	load_bib: (bib) => @bib = { k, BibItem k, v for k,v in pairs bib }
	read: (raw_src='bib.yml') =>
		if not @has_src
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
			if not succ
				attempts = ''
				if #acceptible_srcs > 1
					attempts = ", tried: #{concat [ "'#{f}'" for f in *acceptible_srcs ], ', '}"
				log_warn "Could not find bibliography file '#{src}'#{attempts}"

			@load_bib load lines
			@has_src = true
	output: =>
		included_bib = sorted [ itm for ref,itm in pairs @bib when @contents[ref] ]
		itm\set_bib_idx i for i,itm in pairs included_bib

		bib_table = Content [ (Word itm\cite!) .. itm.author .. (it itm.title) .. itm.year for itm in *included_bib ]
		(Call 'h1*', 'Bibliography') .. bib_table

bib = Bib!
em.bib = (src) ->
	if em_iter == 1
		bib\read src
	bib\output!
em.cite = (ref) -> bib\add ref

{ :Bib, :get_cite_style, :set_cite_style, :cite_styles }
