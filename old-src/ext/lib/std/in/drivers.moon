---
-- @file std.in.drivers
-- @brief Manages input drivers, heeps associations for mapping input languages to parsers, and file extensions to input languages
-- @author Edward Jones
-- @date 2021-09-17

import Directive, em, eval_string, include_file from require 'std.base'
import unknown_x_msg from require 'std.edit'
import key_list from require 'std.func'
import log_err_here from require 'std.log'

import __em from _G
import __include_file from __em

---
-- @brief Holds a map of known languages to input & parser functions
known_languages =
	em: __include_file

---
-- @brief Holds a map of known file extensions to input languages
known_file_extensions =
	em: 'em'

parse_results = {}

parse_file = (f, language) ->
	f = eval_string f
	if f == nil or f == ''
		log_err_here "Nil or empty file name given"
	if language != nil
		language = eval_string language
	elseif extension = f\match '.*%.(.*)'
		language = known_file_extensions[extension]
		if language == nil
			log_err_here unknown_x_msg 'file extension', extension, key_list known_file_extensions
	else
		language = 'em'
	if parser = known_languages[language]
		return parser f

	log_err_here unknown_x_msg 'parsing language', language, key_list known_languages

em.include = Directive 1, 1, "Parse a given source file with an optional language. If no language is given, if there is one, the file extension is checked against known extensions, otherwise 'em' is assumed. The result is cached, so each file is only parsed once.", (f, ...) ->
	f = eval_string f
	local ret
	unless ret = parse_results[f]
		ret = parse_file f, ...
		parse_results[f] = ret
	ret
em['include*'] = Directive 1, 1, "Parse a given source file with an optional language. If no language is given, if there is one, the file extension is checked against known extensions, otherwise 'em' is assumed.", parse_file

{ :known_languages, :known_file_extensions }
