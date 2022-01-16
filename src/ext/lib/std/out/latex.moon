---
-- @file std.out.latex
-- @brief Provides an output driver for [LaTeX](https://www.latex-project.org)
-- @author Edward Jones
-- @date 2021-09-24

import node_string from require 'std.base'
import bib from require 'std.bib'
import css, driver_capabilities from require 'std.constants'
import log_err, log_warn from require 'std.log'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'
import colour_to_hex from require 'std.style'
import eq, extend, is_list, sorted, StringBuilder from require 'std.util'
import concat, sort from table

local open
import open from io unless io.module_unavailable

import TS_CSS_STYLES from driver_capabilities
import FONT_FAMILY_MONOSPACE, FONT_FAMILY_SANS_SERIF, FONT_FAMILY_SERIF from css.font_family
import FONT_STYLE_ITALIC, FONT_STYLE_OBLIQUE from css.font_style
import FONT_WEIGHT_BOLD, FONT_WEIGHT_BOLDER, FONT_WEIGHT_700, FONT_WEIGHT_800, FONT_WEIGHT_900 from css.font_weight
import FONT_VARIANT_SMALL_CAPS from css.font_variant
import TEXT_ALIGN_CENTRE, TEXT_ALIGN_LEFT, TEXT_ALIGN_RIGHT, TEXT_ALIGN_JUSTIFY from css.text_align

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
		"\\usepackage#{opt_string}{#{@name}}\n"

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
	new: (do_wrap_root) => super do_wrap_root, '~', TS_CSS_STYLES, 'tex'
	output: (doc, use_stdout, @stem, @generation_time) =>
		@bib_loc = "#{@stem}.bib"
		if not open
			log_warn "Extension-space output drivers unavailable due to sandbox level"
			return

		@output_bib @bib_loc, use_stdout
		super doc, use_stdout, @stem, @generation_time
	output_bib: (bib_loc, use_stdout) =>
		records = [ LaTeXBibRecord k, v for k,v in pairs bib\records! ]
		sort records, (a,b) -> a.name < b.name
		formatted_bib = concat [ tostring r for r in *records ], '\n\n'

		if use_stdout
			print formatted_bib
		else
			f = open bib_loc, 'w'
			log_err "Failed to open file #{bib_loc}" unless f
			with f
				\write formatted_bib
				\close!
	par_inner_sep: '\n'
	special_tag_map:
		ul: 'itemize'
		ol: 'enumerate'
		bib: 'bibliography'
		cite: 'cite'
		h1: 'section'
		h2: 'subsection'
		h3: 'subsubsection'
		h4: 'subsubsubsection'
		'h1*': 'section*'
		'h2*': 'subsection*'
		'h3*': 'subsubsection*'
		'h4*': 'subsubsubsection*'
	raw_tags:
		cite: true
	const_tags:
		bibliography: => @bib_loc
	environments:
		itemize: true
		enumerate: true
		FlushLeft: true
		FlushRight: true
		Center: true
		justify: true
	style_responses: {
		=> 'textit' if @font_style == FONT_STYLE_ITALIC
		=> 'textsl' if @font_style == FONT_STYLE_OBLIQUE
		=>
			fw = @font_weight
			'textbf' if fw == FONT_WEIGHT_BOLD or
				fw == FONT_WEIGHT_BOLDER or
				fw == FONT_WEIGHT_700 or
				fw == FONT_WEIGHT_800 or
				fw == FONT_WEIGHT_900
		=> 'texttt' if @font_family.type == FONT_FAMILY_MONOSPACE
		=> 'textsf' if @font_family.type == FONT_FAMILY_SANS_SERIF
		=> 'textrm' if @font_family.type == FONT_FAMILY_SERIF
		=> 'textsc' if @font_variant == FONT_VARIANT_SMALL_CAPS
		=> 'Center' if @text_align == TEXT_ALIGN_CENTRE
		=> 'justify' if @text_align == TEXT_ALIGN_JUSTIFY
		=> 'FlushLeft' if @text_align == TEXT_ALIGN_LEFT
		=> 'FlushRight' if @text_align == TEXT_ALIGN_RIGHT
		=> if 'table' == type @colour
			hex = colour_to_hex @colour
			"definecolor{color@#{hex}}{HTML}{#{hex}}\\textcolor{color@#{hex}}"
	}
	special_tag_enclose: (t, r, as) =>
		if @environments[t]
			return { '\\begin{', t, '}%\n\t', r, '%\n\\end{', t, '}' }
		if @raw_tags[t]
			r = node_string as[1]
		else if f = @const_tags[t]
			r = f @, as
		{ '{\\', t, '{', r, '}}' }
	default_libs: {
		LaTeXLib 'babel', 'UKenglish'
		LaTeXLib 'microtype', {'kerning','tracking','spacing'}
		LaTeXLib 'graphicx'
		LaTeXLib 'balance'
		LaTeXLib 'amsmath'
		LaTeXLib 'xcolor', 'table'
		LaTeXLib 'geometry', 'a4paper'
		LaTeXLib 'adjustbox'
		LaTeXLib 'ragged2e'
		LaTeXLib 'biblatex', {'backend': 'bibtex'}
	}
	wrap_root: (r) => StringBuilder {
		'\\documentclass{article}\n',
		'\n',
		@default_libs,
		'\n',
		'\\microtypecontext{spacing=nonfrench}\n',
		'\\SetExtraKerning{encoding=*,family=*}{\\textemdash={83,83}}\n',
		'\n',
		'\\begin{document}%\n',
		{ r\get_contents!, '%\n' },
		'\\end{document}'
	}
	sanitise: (w) =>
		w = w\gsub '([\\{}])', '\\%1'
		w = w\gsub '%^', '{\\textasciicircumflex}'
		w = w\gsub '([%%&^_$])', '\\%1'
		w

output_drivers.latex = LaTeXOutputDriver true
output_drivers.latex_bare = LaTeXOutputDriver false

{ :LaTeXOutputDriver }
