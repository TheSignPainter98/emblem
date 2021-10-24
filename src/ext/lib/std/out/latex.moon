---
-- @file std.out.latex
-- @brief Provides an output driver for [LaTeX](https://www.latex-project.org)
-- @author Edward Jones
-- @date 2021-09-24

import css, driver_capabilities from require 'std.constants'
import TextualMarkupOutputDriver, output_drivers from require 'std.out.drivers'
import eq, is_list from require 'std.util'
import extend from require 'std.util'
import concat from table

import TS_BASIC_STYLING, TS_COLOUR, TS_TEXT_SIZE from driver_capabilities
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
		=> "definecolor{color_#{@colour.hex}{HTML}{#{@colour.hex}}\\textcolor{color_#{@colour.hex}}" if 'table' == type @colour
	}
	special_tag_enclose: (t, r) =>
		if @environments[t]
			"\\begin{#{t}}%\n\t#{r}%\n\\end{#{t}}"
		else
			"{\\#{t}{#{r}}}"
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
	}
	wrap_root: (r) =>
		-- There should really be a way of customising this
		libs = [ tostring l for l in *@default_libs ]
		setup = {
			'\\microtypecontext{spacing=nonfrench}'
			'\\SetExtraKerning{encoding=*,family=*}{\\textemdash={83,83}}'
		}
		lines = extend {'\\documentclass{article}'},
			libs,
			setup,
			{'\\begin{document}%'},
			{r .. '%'},
			{'\\end{document}%'}
		concat lines, '\n'
	sanitise: (w) =>
		w = w\gsub '([\\{}])', '\\%1'
		w = w\gsub '%^', '{\\textasciicircumflex}'
		w\gsub '([%%&^_$])', '\\%1'

output_drivers.latex = LaTeXOutputDriver true
output_drivers.latex_bare = LaTeXOutputDriver false

{ :LaTeXOutputDriver }
