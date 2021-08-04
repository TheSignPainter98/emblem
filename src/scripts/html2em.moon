#!/usr/bin/moon

import parse from require 'htmlparser'
import input, stderr from io
import exit from os
import len from string
import unpack from table

name = 'html2em'
version = '1.0.0'
description = 'A simple converter from html to emblem code'
license_text = [=[
Copyright (C) 2021 Edward Jones

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
]=]


concat_list = (as={}, bs={}) ->
	for i = 1,#bs
		as[#as + 1] = bs[i]
	as

get_styling_node_name = (name) ->
	switch name
		when 'b'
			'bf'
		when 'i'
			'it'
		when 'tt'
			'tt'

get_styling_node_names = (name) ->
	snn = get_styling_node_name name
	if snn
		{snn}
	else
		{}

sanitise_em = (c) ->
	if c == ''
		return '// Naught'
	i = 0
	clen = len c
	out = ''
	for chr in c\gmatch '.'
		i += 1
		escape = false
		switch chr
			when '_', '*', '`'
				if i == 1 or i == clen
					escape = true
			when '.', ':'
				if i == 1
					escape = true
			when '{', '}'
				escape = true
		out ..= '\\' if escape
		out ..= chr
	out

print_em = (root, indent_char, indent='') ->
	styling_node_names = get_styling_node_names root.name
	directives = concat_list styling_node_names, root.classes

	for cls in *directives
		print indent .. ".#{cls}:"
		indent ..= indent_char

	if #root.nodes > 0
		for p in *root.nodes
			print_em p, indent_char, indent
	else
		print indent .. sanitise_em root\getcontent!

html_to_em = (fname, indent_char='\t') ->
	if not fname
		stderr\write 'Reading from stdin...\n'
	file = input fname
	root = parse file\read '*a'
	file\close!
	print_em root, indent_char

argument_parser = require 'argparse'
arg_parser = argument_parser!
arg_parser\name 'html2em'
arg_parser\description description

with arg_parser\argument 'file-name'
	\description 'A file name to process'
	\target 'fname'
	\default '-'
with arg_parser\option '-i --indent-char'
	\description 'Character to use when indenting'
	\default '\t'
with arg_parser\flag '-V --version'
	\description 'Output version and exit'
args = arg_parser\parse!

if args.version
	stderr\write "#{name} #{version}\n#{license_text}"
	exit 0

args.fname = nil if args.fname == '-'

html_to_em args.fname, args.indent_char
