---
-- @file std.out.latex
-- @brief Provides an output driver for [LaTeX](https://www.latex-project.org)
-- @author Edward Jones
-- @date 2021-09-24

import node_string from require 'std.base'
import bib from require 'std.bib'
import driver_capabilities from require 'std.constants'
import log_err, log_warn from require 'std.log'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'
import elem, eq, extend, is_list, sorted from require 'std.util'
import concat, sort from table

local open
import open from io unless io.module_unavailable

import TS_BASIC_STYLING, TS_COLOUR, TS_TEXT_SIZE from driver_capabilities

class LaTeXLib
	new: (@name,@opts) =>
	sanitise_string: (s) =>
		return "{#{s}}" if s\match ','
		s
	__tostring: =>
		opt_string = ''
		if @opts
			if 'table' != type @opts
				opt_string = "[#{@opts}]"
			elseif not eq @opts, {}
				if is_list @opts
					opt_string = "[#{concat [ tostring o for o in *@opts ], ','}]"
				else
					opt_string = "[#{concat [ "#{k}=#{v}" for k,v in pairs @opts ], ','}]"
		"\\usepackage#{opt_string}{#{@name}}"

class LaTeXBibRecord
	new: (@name, @map) =>
	__tostring: =>
		vals = sorted [ "#{k} = {#{@sanitise_value v}}" for k,v in pairs @map when k != 'type' ]
		"@#{@map.type or 'article'}{#{@name},\n\t#{concat vals, ',\n\t'}\n}"
	sanitise_value: (v) =>
		v = tostring v if 'string' != type v
		v = v\gsub '([{}&$%%\\])', '\\%1'
		v

---
-- @brief Represents an output driver for LaTeX
class LaTeXOutputDriver extends TextualMarkupOutputDriver
	new: (do_wrap_root) =>
		support = TS_BASIC_STYLING | TS_COLOUR | TS_TEXT_SIZE
		super do_wrap_root, support, 'tex'
	output: (doc, use_stdout, @stem, @generation_time) =>
		@bib_loc = "#{@stem}.bib"
		if not open
			log_warn "Extension-space output drivers unavailable due to sandbox level"
			return

		records = [ LaTeXBibRecord k, v for k,v in pairs bib\records! ]
		sort records, (a,b) -> a.name < b.name
		formatted_bib = concat [ tostring r for r in *records ], '\n\n'

		if use_stdout
			print formatted_bib
		else
			f = open @bib_loc, 'w'
			log_err "Failed to open file #{@bib_loc}" unless f
			with f
				\write formatted_bib
				\close!

		super doc, use_stdout, @stem, @generation_time
	par_inner_sep: '\n'
	special_tag_map:
		ul: 'itemize'
		ol: 'enumerate'
		bib: 'bibliography'
		cite: 'cite'
		it: 'textit'
		sc: 'textsc'
		bf: 'textbf'
		tt: 'texttt'
		af: 'textsf'
		h1: 'section'
		h2: 'subsection'
		h3: 'subsubsection'
		h4: 'subsubsubsection'
		'h1*': 'section*'
		'h2*': 'subsection*'
		'h3*': 'subsubsection*'
		'h4*': 'subsubsubsection*'
	raw_tags: {
		'bibliography'
		'cite'
	}
	environments:
		itemize: true
		enumerate: true
	special_tag_enclose: (t, r, as) =>
		if @environments[t]
			"%\n\n\\begin{#{t}}%\n\t#{r}%\n\\end{#{t}}"
		else
			prefix = ''
			if t\match 'section%*?$'
				if @first_block
					@first_block = false
				else
					prefix = '\n\n'
			p = r
			if t == 'bibliography'
				p = @bib_loc
			else if elem t, @raw_tags
				p = node_string as[1]
			prefix .. "\\#{t}{#{p}}"
	default_libs: => {
		LaTeXLib 'babel', 'UKenglish'
		LaTeXLib 'microtype', {'kerning','tracking','spacing'}
		LaTeXLib 'graphicx'
		LaTeXLib 'balance'
		LaTeXLib 'amsmath'
		LaTeXLib 'xcolor', 'table'
		LaTeXLib 'geometry', 'a4paper'
		LaTeXLib 'adjustbox'
		LaTeXLib 'biblatex', {'backend': 'bibtex'}
	}
	wrap_root: (r) =>
		-- There should really be a way of customising this
		libs = [ tostring l for l in *@default_libs! ]
		setup = {
			'\\microtypecontext{spacing=nonfrench}'
			'\\SetExtraKerning{encoding=*,family=*}{\\textemdash={83,83}}'
		}
		lines = extend {'\\documentclass{article}'},
			libs,
			setup,
			{'\\begin{document}%'},
			{r},
			{'\\end{document}%'}
		concat lines, '\n'


output_drivers.latex = LaTeXOutputDriver true
output_drivers.latex_bare = LaTeXOutputDriver false

{ :LaTeXOutputDriver }
