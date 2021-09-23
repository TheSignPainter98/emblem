---
-- @file std.base
-- @brief Provides the base library for use with extensions
-- @author Edward Jones
-- @date 2021-09-17

import len, lower from string
import concat, insert from table

constants = require 'std.constants'
import WORD, CALL, CONTENT from constants.node_types

collectgarbage 'stop' -- TODO: remove the need for this!

base = { :eval, :include_file, :requires_reiter, :_log_err, :_log_err_at, :_log_warn, :_log_warn_at, :_log_info, :_log_debug, :_em_loc, }
for k,v in pairs constants
	base[k] = v

-- Calling wrap_indices @ in a constructor before the end seems to be able to cause a memory leak.
base.wrap_indices = =>
	mt = getmetatable @

	-- Handle __index
	old_index = mt.__index
	new_index = mt.__get
	call__index = (idx, cls, k) ->
		if 'function' == type idx
			idx cls, k
		else
			idx[k]
	if new_index
		if old_index
			mt.__index = (k) =>
				ret = call__index old_index, @, k
				if ret != nil
					ret
				else
					call__index new_index, @, k
		else
			mt.__index = new_index

	-- Handle __newindex
	mt.__newindex = mt.__set if mt.__set

class UnimplementedLuaStandardModule
	new: (@mod_name) => base.wrap_indices @
	module_unavailable: true
	__tostring: => "Unimplemented module '#{@mod_name}'"
	__get: (k) =>
		error "Module #{rawget @, 'mod_name'} is not available at this sandbox level (trap activated when importing '#{k}')" unless k == 'module_unavailable'
		rawget @, k

if not io
	export io = UnimplementedLuaStandardModule 'io'
if not os
	export os = UnimplementedLuaStandardModule 'os'

class Directive
	new: (@nmand, @nopt, msg_or_func, func) =>
		if func == nil
			@func = msg_or_func
			@msg = '[no help given]'
		else
			@func = func
			@msg = msg_or_func
base.Directive = Directive

class DirectiveHelp
	new: (@dname, @direc) =>
	__tostring: => ".#{@dname}: #{@direc.msg} (takes #{@direc.nmand} mandatory and #{@direc.nopt} optional arguments)"

class SanitisedKeyTable
	new: => base.wrap_indices @
	__tostring: show
	_sanitise_key: (k) -> lower k\gsub '_', '-'
	__get: (k) =>
		k = (rawget (getmetatable @), '_sanitise_key') k
		rawget @, k
	__set: (k, v) =>
		k = (rawget (getmetatable @), '_sanitise_key') k
		rawset @, k, v
base.SanitisedKeyTable = SanitisedKeyTable

help = SanitisedKeyTable!
base.help = help

is_instance = (cls, obj) ->
	return true if cls == type obj
	return false if 'table' != type obj
	mt = getmetatable obj
	return false if mt == nil
	cls = cls.__name if 'table' == type cls
	ocls = mt.__class
	return false if ocls == nil
	while ocls.__name != cls
		ocls = ocls.__parent
		return false if ocls == nil
	return ocls != nil and ocls.__name == cls
base.is_instance = is_instance

class DirectivePublicTable
	new: => base.wrap_indices @
	__tostring: show
	_sanitise_key: (k) -> lower k\gsub '_', '-'
	__get: (k) =>
		k = (rawget (getmetatable @), '_sanitise_key') k
		rawget @, k
	__set: (k, v) =>
		error "Failed to declare directive #{k}, value is not an instance of Directive" if not is_instance 'Directive', v
		k = (rawget (getmetatable @), '_sanitise_key') k
		if v == nil
			rawset @, k, nil
			help[k] = nil
		wrapped_func = (...) ->
			nargs = select '#', ...
			if nargs < v.nmand
				_log_warn "Directive .#{k} requires at least #{v.nmand} arguments"
			elseif v.nopt > 0 and nargs > v.nmand + v.nopt
				_log_warn "Directive .#{k} takes between #{v.nmand} and #{v.nmand + v.nopt} arguments"
			v.func ...
		rawset @, k, wrapped_func
		help[k] = DirectiveHelp k, v

export em = DirectivePublicTable!
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

em.help = Directive 1, 0, "Show documentation for a given directive", (dname) ->
	dname = eval_string dname
	if ret = help[eval_string dname]
		tostring ret

base.iter_num = -> em_iter

vars = {{}}
base.vars = vars
export open_var_scope = -> insert vars, {}
base.open_var_scope = open_var_scope
export close_var_scope = -> vars[#vars] = nil
base.close_var_scope = close_var_scope

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
em.get_var = Directive 1, 0, "Get the value of a variable in the current scope", get_var

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
em.set_var = Directive 2, 0, "Set the value of a variable in the current scope", (n, v) -> set_var_string n, v, true

base.em_loc = -> get_var 'em_loc'
base.copy_loc = -> _copy_loc base.em_loc!

base
