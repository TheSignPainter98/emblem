import stderr from io
import lower, match from string
import concat, insert from table
import Content from require 'std.ast'
import keys from require 'std.func'
import em, eval_string, get_var, include_file, set_var, set_var_string from require 'std.base'

em.def = (n, f) -> em[eval_string n] = f
em['undef-dir'] = (n) -> em[eval_string n] = nil

em.echo = (...) ->
	print concat [ eval_string v for v in *{...} when v != nil ], ' '

em['echo-on-pass'] = (n, ...) ->
	n = tonumber eval_string n
	if em_iter == n
		em.echo ...

cond = (c) ->
	if not c
		return false
	r = eval_string c
	if '' == r or '0' == r or 'false' == lower r
		false
	else
		true

toint = (b) ->
	if b
		1 else 0

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

em.include = (f, language='em') ->
	f = eval_string f
	language = eval_string language
	if parser = known_languages[language]
		return parser f
	known_languages_str = concat [ "- #{k}" for k in keys known_languages ], '\n\t'
	error "Unknown parsing language '#{language}', currently known languages:\n\t#{known_languages_str}\nPerhaps there's a typo or missing input driver import?"
	nil


{:cond, :toint, :known_languages }
