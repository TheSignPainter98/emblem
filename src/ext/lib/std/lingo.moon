import len, lower, match from string
import concat, insert from table
import eval_string from require 'std.base'

vars = {{}}

get_scope_widening = (vn) ->
	w = len vn\match "^!*"
	w += 1 if w
	w

get_var = (rn, d) ->
	wn = eval_string rn
	widen_by = get_scope_widening wn
	n = wn\match "[^!]*$"
	for i = #vars - widen_by, 1, -1
		v = vars[i][n]
		if v != nil
			return v
	d
em['get-var'] = get_var

set_var = (n, v) ->
	vars[#vars - 1][eval_string n] = eval_string v
em['set-var'] = set_var

export open_var_scope = -> insert vars, {}
export close_var_scope = -> vars[#vars] = nil

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
		insert ret, eval b
	ret

em.foreach = (n, vs, b) ->
	ret = {}
	n = eval_string n
	prev_val = vars[n]
	for v in (eval_string vs)\gmatch('%S+')
		set_var n, v
		insert ret, eval b
	setvar n, prev_val
	ret

em.streq = (s, t) ->
	toint (eval_string s) == eval_string t

em.defined = (v) ->
	toint vars[v] != nil

em.exists = (f) ->
	toint em[f] != nil

em.include = (f) -> include_file eval_string f

{:cond, :toint, :set_var, :get_var}
