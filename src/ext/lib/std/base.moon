import len from string
import concat, insert from table

collectgarbage 'stop' -- TODO: remove the need for this!

base = { :eval, :include_file, :node_types, :requires_reiter, :_log_err, :_log_err_at, :_log_warn, :_log_warn_at, :_log_info, :_log_debug, :_em_loc }

class PublicTable
	__tostring: show
export em = PublicTable!
base.em = em

node_string = (n) ->
	if n == nil
		return nil
	if 'table' != type n
		return tostring n
	switch n.type
		when node_types.word
			return n.word
		when node_types.call
			return node_string n.result
		when node_types.content
			return concat [ node_string w for w in *n.content when w != nil ], ' '
		else
			error "Unrecognised node type '#{n.type}'"
			return 1
base.node_string = node_string

eval_string = (d) ->
	if 'userdata' == type d
		return node_string eval d
	tostring d
base.eval_string = eval_string

base.iter_num = -> em_iter

vars = {{}}
base.vars = vars
export open_var_scope = -> insert vars, {}
export close_var_scope = -> vars[#vars] = nil

get_scope_widening = (vn) ->
	w = len vn\match "^!*"
	w += 1 if w
	w

export get_var = (rn, d) ->
	wn = eval_string rn
	widen_by = get_scope_widening wn
	n = wn\match "[^!]*$"
	for i = #vars - widen_by, 1, -1
		v = vars[i][n]
		if v != nil
			return v
	d
base.get_var = get_var
em['get-var'] = get_var

export set_var = (n, v) ->
	vars[#vars - 1][eval_string n] = v
base.set_var = set_var

set_var_string = (n, v) -> set_var n, eval_string v
base.set_var_string = set_var_string
em['set-var'] = set_var_string

base.em_loc = -> get_var 'em_loc'
base.copy_loc = -> _copy_loc base.em_loc!

base
