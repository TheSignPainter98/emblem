#!/usr/bin/env moon

import open from io
import load from require 'lyaml'
import sort from table
ArgParser = require 'argparse'

-- Parse arguments
arg_parser = with ArgParser!
	\name 'compgen'
	\description 'Shell script completions'
	with \argument 'language'
		\description 'The language of the completion script'
		\default 'bash'
args = arg_parser\parse!

arg_text = => @short and @long and "#{@short} #{@long}" or @metaDest

-- Import the spec
local spec
with open 'em.yml'
	spec = load \read '*all'
	\close!
sort spec.args, (a,b) ->
	san = (x) -> (arg_text x)\lower!
	(san a) < san b

-- Parse spec
em_imposter_arg_parser = with ArgParser!
	\name spec.program
	\description spec.description

	get_imp_arg = (arg, atxt) ->
		atxt = arg_text arg
		switch arg.type
			when 'help'
				\add_help atxt
			when 'version', 'flag'
				\flag atxt
			when 'int', 'char*', 'List', 'double'
				if arg.short and arg.long
					\option atxt
				else
					\argument atxt
			else
				print "Unknown argspec type #{arg.type}"

	for arg in *spec.args
		imp_arg = get_imp_arg arg
		imp_arg\description arg.help
		if arg.default
			imp_arg\default arg.default
		if arg.choices
			imp_arg\choices [ tostring c for c in *arg.choices ]
		else
			switch arg.type
				when 'flag', 'int'
					imp_arg\choices {}
	\add_complete!

-- Output completion script
em_imposter_arg_parser\parse { '--completion', args.language }
