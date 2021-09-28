---
-- @file std.lingo
-- @brief Provides a rudamentary scripting language for use in documents
-- @author Edward Jones
-- @date 2021-09-17

import lower, match from string
import concat, insert from table
import Call, Content from require 'std.ast'
import close_var_scope, Directive, em, eval, eval_string, get_var, open_var_scope, set_var, set_var_string, vars from require 'std.base'
import node_flags from require 'std.constants'
import expr from require 'std.expr'
import key_list from require 'std.func'
import log_debug, log_err_here, log_warn_here from require 'std.log'
import eq, on_iter_wrap, sorted from require 'std.util'

import NO_FURTHER_EVAL from node_flags

local getenv, popen
unless os.module_unavailable
	import getenv from os
unless io.module_unavailable
	import popen from io

em.known_directives = Directive 0, 0, "Return a list of known directives", -> concat (sorted key_list em), ' '

em.def = Directive 2, 0, "Takes a name and a section of document and creates a directive of the same name", (n, f) -> em[eval_string n] = Directive 0, -1, (...) ->
	nargs = select '#', ...
	set_var '#', nargs
	set_var i, (select i, ...) for i=1,nargs
	f
em.undef = Directive 1, 0, "Undefine a directive", (n) -> em[eval_string n] = nil

em.echo = Directive 0, -1, "Output text to stdout", (...) ->
	print concat [ eval_string v for v in *{...} when v != nil ], ' '

em['echo-on'] = Directive 1, -1, "Output text to stdout on a given iteration", on_iter_wrap em.echo

em.call = Directive 1, -1, "Takes a directive and constructs a call to it with the remainder of the given arguments", (d, ...) ->
	Call (eval_string d), {...}

---
-- @brief Takes input of value and evaluates it as a condition
-- @param c Lua value or core pointer to evaluate as a condition
-- @return false if `c` is `false` or is the empty string, '0' or 'false' (case-insensitive), otherwise true
cond = (c) ->
	if not c
		return false
	r = eval_string c
	return false if r == ''

	e = expr r
	if e == 0 or not e
		false
	else
		true

---
-- @brief Converts a value to a condition integer (for a more compact representation
-- @param b A value to check
-- @return 0 if b is 0, the empty string, false or nil, otherwise 1
toint = (b) ->
	return 0 if b == 0 or b == '' or not b
	1

em.if = Directive 3, 0, "Return one of two branches depending on the value of a condition", (c, b, e) ->
	if cond c
		b
	else
		e

specials = { 1/0, -1/0, 0/0, -0/0 }
special_strings = [ tostring v for v in *specials ]
safe_tonumber = (n) ->
	ns = eval_string n
	for i=1,#special_strings
		if ns == special_strings[i]
			return specials[i]
	n = tonumber ns
	if n == nil
		log_warn_here "Failed to parse number"
	n or 0

em.case = Directive 2, -1, "Takes a number, n, and a list of branches and returns the nth branch if there is one, otherwise the last", (n, ...) ->
	n = safe_tonumber n
	if 1 <= n and n <= select '#', ...
		select n, ...
	else
		select (select '#', ...), ...

em.not = Directive 1, 0, "Inverts a boolean value", (b) -> toint not cond b
em.all = Directive 0, -1, "Takes some conditions, returns true if they are all true. This is lazy, so will only evaluate as many conditions from the left are required to confirm the result", (...) ->
	return toint false for c in *{...} when not cond c
	return toint true
em.any = Directive 0, -1, "Takes some conditions, returns true if any of them is true. This is lazy, so will only evaluate as many conditions from the left are required to confirm the result", (...) ->
	return toint true for c in *{...} when cond c
	return toint false
em.impl = Directive 2, 0, "Takes two conditions, returns true if one implies the other, that is, false if the first is true but the second is false, otherwise true", (c1, c2) -> toint (not cond c1) or cond c2
em.xor = Directive 2, 0, "Takes two conditions, returns true if either is true but not both", (c1, c2) ->
	c1 = cond c1
	c2 = cond c2
	return toint false if c1 and c2
	toint c1 or c2

em['$'] = Directive 1, 0, "Execute a shell command", (cmd) ->
	unless popen
		log_warn_here "Sub-process opening is restricted at this sandbox level"
		return nil
	cmd = eval_string cmd
	local result
	time = -os.clock!
	with popen cmd, 'r'
		result = \read '*all'
		passed, mode, rc = \close!
		time += os.clock!
		unless passed
			switch mode
				when 'exit'
					log_err_here "Command '#{cmd}' failed with exit code #{rc}"
				when 'signal'
					log_err_here "Command '#{cmd}' was killed by signal #{rc}"
	log_debug "It took #{time}s to run #{cmd}"
	result

em.env = Directive 1, 0, "Get the value of an environment variable", (var) ->
	unless getenv
		log_warn_here "Cannot interact with env at this sandbox level"
		return nil
	getenv eval_string var

em.lt = Directive 2, 0, "Checks whether the value of the left is less than that of the right", (l, r) -> toint (safe_tonumber l) < safe_tonumber r
em.le = Directive 2, 0, "Checks whether the value of the left is less than or equal that of the right", (l, r) -> toint (safe_tonumber l) <= safe_tonumber r
em.gt = Directive 2, 0, "Checks whether the value of the left is greater than that of the right", (l, r) -> toint (safe_tonumber l) > safe_tonumber r
em.ge = Directive 2, 0, "Checks whether the value of the left is greater than or equal that of the right", (l, r) -> toint (safe_tonumber l) >= safe_tonumber r
em.eq = Directive 2, 0, "Checks whether two values are equal", (l, r) -> toint eq (eval l), eval r
em.numeq = Directive 2, 0, "Checks whether two numbers are equal", (l, r) -> toint (safe_tonumber l) == safe_tonumber r
em.streq = Directive 2, 0, "Extract text from two trees, returns whether they are equal", (s, t) ->
	toint (eval_string s) == eval_string t

em.abs = Directive 1, 0, "Add two numbers together and return their result", (n) ->
	n = safe_tonumber n
	return -n if n < 0
	n
em.sign = Directive 1, 0, "Returns 1 for positive numbers -1 for negative ones otherwise 0", (n) ->
	n = safe_tonumber n
	return 1 if n > 0
	return -1 if n < 0
	0

em.add = Directive 2, 0, "Add two numbers together and return their result", (a, b) -> (safe_tonumber a) + (safe_tonumber b)
em.sub = Directive 2, 0, "Subtract one number from another and return their result", (a, b) -> (safe_tonumber a) - (safe_tonumber b)
em.mul = Directive 2, 0, "Take the product of two numbers", (a, b) -> (safe_tonumber a) * (safe_tonumber b)
em.div = Directive 2, 0, "Take the product of two numbers", (a, b) -> (safe_tonumber a) / (safe_tonumber b)
em.idiv = Directive 2, 0, "Take the product of two numbers", (a, b) ->
	numer = (safe_tonumber a)
	denom = (safe_tonumber b)
	return numer // denom if denom != 0
	numer / denom
em.mod = Directive 2, 0, "Take the modulo of two numbers", (a, b) ->
	dividend = (safe_tonumber a)
	divisor = (safe_tonumber b)
	return dividend / divisor if divisor == 0
	dividend % divisor
em.pow = Directive 2, 0, "Take the modulo of two numbers", (a, b) -> (safe_tonumber a) ^ (safe_tonumber b)

em.while = Directive 2, 0, "Takes a condition and a body, repeats the body until the condition no longer holds", (c, b) ->
	ret = {}
	while cond c
		open_var_scope!
		insert ret, eval b
		close_var_scope!
	Content ret, NO_FURTHER_EVAL

em.for = Directive 4, 0, "Takes an initialiser, condition, updater and body, executes the initialiser, then repeatedly: evaluates the condition, executes the body and then the updator, until the condition is false.", (i, c, m, b) ->
	ret = {}
	eval i
	while cond c
		open_var_scope!
		insert ret, eval b
		close_var_scope!
		eval m
	Content ret, NO_FURTHER_EVAL

em.foreach = Directive 3, 0, "Takes a variable name, a list of values and a body, repeats the body with the variable taking each value specified, in the order given", (n, vs, b) ->
	ret = {}
	n = eval_string n
	for v in (eval_string vs)\gmatch '%S+'
		open_var_scope!
		set_var_string n, v
		insert ret, eval b
		close_var_scope!
	Content ret, NO_FURTHER_EVAL

em.exists = Directive 1, 0, "Checks whether a given variable exists", (v) ->
	toint vars[v] != nil

em.defined = Directive 1, 0, "Checks whether a given directive has been defined", (f) ->
	toint em[f] != nil

{:cond, :toint }
