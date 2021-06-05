import lower from string
import concat, insert from table
import eval_string from require 'std.std'

vars = {}

get_var = (rn, d) ->
	n = eval_string rn
	if vars[n]
		return vars[n]
	return d
em['get-var'] = get_var

set_var = (n, v) ->
	vars[eval_string n] = eval_string v
em['set-var'] = set_var

em.echo = (...) ->
	print concat [ eval_string v for v in *{...} ], ' '

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

em.if = (c, b, e) ->
	if cond c
		b
	else
		e

em.while = (c, b) ->
	ret = {}
	while cond c
		insert ret, eval b
	ret

em.foreach = (v, vs, b) ->
	ret = {}
	for e in (eval_string vs)\gmatch('%S+')
		set_var e, v
		insert ret, eval b
	ret

em.streq = (s, t) ->
	toint (eval_string s) == eval_string t

em.defined = (v) ->
	toint vars[v] != nil

em.exists = (f) ->
	toint em[f] != nil

{:cond, :toint}
