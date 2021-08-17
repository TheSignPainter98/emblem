import len from string
import concat, insert from table

constants = require 'std.constants'
import WORD, CALL, CONTENT from constants.node_types

collectgarbage 'stop' -- TODO: remove the need for this!

base = { :eval, :include_file, :requires_reiter, :_log_err, :_log_err_at, :_log_warn, :_log_warn_at, :_log_info, :_log_debug, :_em_loc, }
for k,v in pairs constants
	base[k] = v

class PublicTable
	__tostring: show
export em = PublicTable!
base.em = em

node_string = (n) ->
	if n == nil
		return nil
	if 'table' != type n or ('table' == type n and n.type == nil)
		return tostring n
	switch n.type
		when WORD
			return n.word
		when CALL
			return node_string n.result
		when CONTENT
			ss = {}
			for m in *n.content
				if s = node_string m
					insert ss, s
			return concat ss, ' '
		else
			error "Unrecognised node type '#{n.type}'"
			return nil
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
	local idx
	if #vars > 1
		idx = #vars - 1
	else
		idx = 1
	vars[idx][eval_string n] = v
base.set_var = set_var

set_var_string = (n, v) -> set_var n, eval_string v
base.set_var_string = set_var_string
em['set-var'] = set_var_string

base.em_loc = -> get_var 'em_loc'
base.copy_loc = -> _copy_loc base.em_loc!

base
