import stderr from io
import lower, match from string
import concat, insert from table
import Content from require 'std.ast'
import em, eval, eval_string, get_var, include_file, set_var, set_var_string, vars from require 'std.base'
import unknown_x_msg from require 'std.edit'
import keys, key_list from require 'std.func'
import log_err_here from require 'std.log'
import on_iter_wrap, sorted from require 'std.util'

em.known_directives = -> concat (sorted key_list em), ' '

em.def = (n, f) -> em[eval_string n] = f
em.undef = (n) -> em[eval_string n] = nil

em.echo = (...) ->
	print concat [ eval_string v for v in *{...} when v != nil ], ' '

em['echo-on'] = on_iter_wrap em.echo

cond = (c) ->
	if not c
		return false
	r = eval_string c
	if '' == r or '0' == r or 'false' == lower r
		false
	else
		true

toint = (b) ->
	return 0 if b == 0 or b == '' or not b
	1

em.if = (c, b) ->
	if cond c
		b

em.ifelse = (c, b, e) ->
	if cond c
		b
	else
		e

em.case = (n, ...) ->
	n = tonumber eval_string n
	if 1 <= n and n <= #{...}
		select n, ...
	else
		select (select '#', ...), ...

em.while = (c, b) ->
	ret = {}
	while cond c
		insert ret, b
	Content ret

em.foreach = (n, vs, b) ->
	ret = {}
	n = eval_string n
	prev_val = get_var n
	for v in (eval_string vs)\gmatch('%S+')
		set_var_string n, v
		insert ret, eval b
	set_var n, prev_val
	Content ret

em.streq = (s, t) ->
	toint (eval_string s) == eval_string t

em.defined = (v) ->
	toint vars[v] != nil

em.exists = (f) ->
	toint em[f] != nil

known_languages =
	em: include_file

known_file_extensions =
	em: 'em'

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

parse_results = {}
em.include = (f, ...) ->
	f = eval_string f
	local ret
	unless ret = parse_results[f]
		ret = parse_file f, ...
		parse_results[f] = ret
	ret
em['include*'] = parse_file

{:cond, :toint, :known_languages, :known_file_extensions }
