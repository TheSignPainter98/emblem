---
-- @file std.base
-- @brief Provides the base library for use with extensions
-- @author Edward Jones
-- @date 2021-09-17

import len, lower from string
import concat, insert from table

import node_types from require 'std.constants'
import GLUE_LEFT from node_flags
import WORD, CALL, CONTENT from node_types

collectgarbage 'stop' -- TODO: remove the need for this!

base = { :eval, :include_file, :requires_reiter, :_log_err, :_log_err_at, :_log_warn, :_log_warn_at, :_log_info, :_log_debug, :_em_loc, :stylesheet, :em_config_file, :__em_arguments }

stylesheet 'std/base.scss'

---
-- @brief Wrap the __get and __set methods into the __index and __newindex methods if available
-- Calling wrap_indices @ in a constructor before the end seems to be able to cause a memory leak.
-- @param object (table) to wrap
-- @return nil
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

---
-- @brief Stores the necessary information for a directive which may be called
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

---
-- @brief Represents a table which makes no distinction between upper/lower case and _/- in its keys
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

---
-- @brief Tests whether an object is an instance of a given class
-- @param cls The class to test, may be a class name or a class itself
-- @param obj The object to test
-- @return `true` if `obj` is an instance of a sub-class of `cls`, otherwise `false`
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
---
-- @brief Stores directive functions, this table is indexed when evaluating directives to see whether any lua code is to be executed.
base.em = em

---
-- @brief Extracts the text beneath a given node
-- @param n The node to convert into a string, must be a table
-- @param pretty Whether use the pretty or raw form of words
-- @return The text stored at and under the given node
node_string = (n, pretty=false) ->
	str_parts = {}
	node_string_parts = (n) ->
		if n == nil
			return
		if 'table' != type n
			insert str_parts, tostring n
		switch n.type
			when WORD
				insert str_parts, pretty and n.pword or n.word
			when CALL
				node_string_parts n.result
			when CONTENT
				cs = n.content
				cn = #cs
				return if cn == 0
				node_string_parts cs[1]
				for i=2,cn
					m = cs[i]
					insert str_parts, ' ' if (m.flags & GLUE_LEFT) == 0
					node_string_parts m
			when nil
				insert str_parts, tostring n
			else
				error "Unrecognised node type '#{n.type}'"
	node_string_parts n
	concat str_parts
base.node_string = node_string

---
-- @brief Evaluates a node pointer and extracts the text contained in and below it
-- @param d The userdata pointer to evaluate and extract from
-- @param pretty Whether use the pretty or raw form of words
-- @return A string which represents all text at and beneath _d_
eval_string = (d, pretty) ->
	if 'userdata' == type d
		return node_string (eval d), pretty
	tostring d
base.eval_string = eval_string

em.help = Directive 1, 0, "Show documentation for a given directive", (dname) ->
	dname = eval_string dname
	if ret = help[eval_string dname]
		tostring ret

---
-- @brief Returns the number of the current iteration of the typesetting loop (starts at 1)
-- @return The number of times the typesetting loop has been started this run
base.iter_num = -> em_iter

vars = {{}}
---
-- @brief Stores scopes and their contained variables
base.vars = vars

---
-- @brief Opens a new variable scope
export open_var_scope = -> insert vars, {}
base.open_var_scope = open_var_scope

---
-- @brief Closes the most recently-opened variable scope
export close_var_scope = -> vars[#vars] = nil
base.close_var_scope = close_var_scope

get_scope_widening = (n) -> len n\match '^!*'

---
-- @brief Gets the value of a given variable, if the variable name starts with _n_ > 0 exclamation marks, then that many possible matches are skipped while searching from the innermost scope
-- @param rn The raw variable name as a string or core pointer
-- @param d An optional default value to return if `rn` does not exist
-- @return The value of variable `rn` in the current scope otherwise `d`
export get_var = (rn, d) ->
	wn = eval_string rn
	widen_by = get_scope_widening wn
	n = wn\sub 1 + widen_by
	for i = #vars, 1, -1
		v = vars[i][n]
		if v != nil
			if widen_by == 0
				return v
			widen_by -= 1
	d
base.get_var = get_var
em.get_var = Directive 1, 0, "Get the value of a variable in the current scope", get_var

---
-- @brief Set a variable to a given value, if the variable name starts with _n_ > 0 exclamation marks, then a search is performed to set the _n_-th variable with the same name in found whilst searching parent scopes.
-- @param n The name of the variable (string or code pointer)
-- @param v The value to set (not changed by this operation)
-- @param surrounding_scope If set to true, search is bumped up one scope (useful for the .set-var directive which would otherwise have the set value swallowed in its own scope)
export set_var = (n, v, surrounding_scope=false, search=false) ->
	-- If widening, search for parent scopes
	wn = eval_string n
	name_widen = get_scope_widening wn
	n = wn\sub 1 + name_widen
	extra_widen = surrounding_scope and 1 or 0

	if name_widen != 0 or search
		widen_by = name_widen + extra_widen
		for i = #vars, 1, -1
			ve = vars[i][n]
			if ve != nil
				if widen_by == 0
					vars[i][n] = v
					return
				widen_by -= 1
		vars[1][n] = v
	else
		idx = #vars > 1 and #vars - extra_widen or 1
		vars[idx][n] = v
base.set_var = set_var

---
-- @brief Set a given variable to a given value as a string
-- @param n Variable name as for `set_var`
-- @param v Value to evaluate then set to _n_
-- @param w Scope widening paramerer as for `set_var`
set_var_string = (n, v, ...) -> set_var n, (eval_string v), ...
base.set_var_string = set_var_string
em.set_var = Directive 2, 0, "Set the value of a variable in the current scope", (n, v) -> set_var_string n, v, true
em.find_set_var = Directive 2, 0, "Set the value of a variable in the current scope", (n, v) -> set_var_string n, v, true, true

---
-- @brief Get the current location in the source code
-- @return A pointer to the current source location
base.em_loc = -> get_var 'em_loc'

---
-- @brief Copy a source-code location
-- @return A copy of the current source code location
base.copy_loc = -> _copy_loc base.em_loc!

base
