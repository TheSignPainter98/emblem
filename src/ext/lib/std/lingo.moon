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

undef_var = (n) ->
	vars[eval_string n] = nil
em['undef-var'] = undef_var

em.echo = (...) ->
	print concat [ eval_string v for v in *{...} ], ' '

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
		insert ret, eval b
	ret

em.foreach = (n, vs, b) ->
	ret = {}
	n = eval_string n
	prev_val = vars[n]
	for v in (eval_string vs)\gmatch('%S+')
		set_var n, v
		insert ret, eval b
	vars[n] = prev_val
	ret

em.streq = (s, t) ->
	toint (eval_string s) == eval_string t

em.defined = (v) ->
	toint vars[v] != nil

em.exists = (f) ->
	toint em[f] != nil

{:cond, :toint, :set_var, :get_var}
