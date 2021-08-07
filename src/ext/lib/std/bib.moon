import eq, eval_string, sorted, stylers, SyncSet from require 'std.base'
import concat, sort from table
import load from require 'lyaml'

import it from stylers

-- stylesheet 'share/bib.scss'

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
		included_bib = [ itm for ref,itm in pairs @bib when @contents[ref] ]

		sort included_bib, (i1, i2) ->
			fields = { 'author', 'title', 'year' }
			for field in *fields[,#fields - 1]
				if i1[field] != i2[field]
					return i1[field] < i2[field]
			return i1[fields[#fields]] < i2[fields[#fields]]

		bib_table = Content [ (Word "[#{ref}]") .. itm.author .. (it itm.title) .. itm.year for ref,itm in pairs @bib ]
		(h1 'Bibliography') .. bib_table

bib = Bib!
em.bib = (src) ->
	if em_iter == 1
		bib\read src
	bib\output!
em.cite = (ref) ->
	bib\add eval_string ref

{:Bib}
