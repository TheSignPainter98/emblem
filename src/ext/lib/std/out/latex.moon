---
-- @file std.out.latex
-- @brief Provides an output driver for [LaTeX](https://www.latex-project.org)
-- @author Edward Jones
-- @date 2021-09-24

import driver_capabilities from require 'std.constants'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'
import eq, is_list from require 'std.util'
import extend from require 'std.util'
import concat from table

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
					opt_string = "[#{concat [ "#{k}=#{v}" for k,v in ipairs @opts ], ','}]"
		"\\usepackage#{opt_string}{#{@name}}"

---
-- @brief Represents an output driver for LaTeX
class LaTeXOutputDriver extends TextualMarkupOutputDriver
	new: (do_wrap_root) =>
		support = TS_BASIC_STYLING | TS_COLOUR | TS_TEXT_SIZE
		super do_wrap_root, support, 'tex'
	par_inner_sep: '\n'
	special_tag_map:
		ul: 'itemize'
		ol: 'enumerate'
		-- cite: 'cite'
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
	environments:
		itemize: true
		enumerate: true
	special_tag_enclose: (t, r) =>
		if @environments[t]
			"%\n\n\\begin{#{t}}%\n\t#{r}%\n\\end{#{t}}"
		else
			prefix = ''
			if t\match 'section%*?$'
				if @first_block
					@first_block = false
				else
					prefix = '\n\n'
			prefix .. "\\#{t}{#{r}}"
	default_libs: => {
		LaTeXLib 'babel', 'UKenglish'
		LaTeXLib 'microtype', {'kerning','tracking','spacing'}
		LaTeXLib 'graphicx'
		LaTeXLib 'balance'
		LaTeXLib 'amsmath'
		LaTeXLib 'xcolor', 'table'
		LaTeXLib 'geometry', 'a4paper'
		LaTeXLib 'adjustbox'
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
