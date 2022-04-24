---
-- @file std.util
-- @brief Provides miscellaneous utility functions
-- @author Edward Jones
-- @date 2021-09-17

import eval_string, iter_num, meta_wrap from require 'std.base'
import wrap, yield from coroutine
import maxinteger, mininteger from math
import len from string
import concat, insert, sort from table

util = {}

---
-- @brief Returns whether a given value represents a list
-- @param l The value to check
-- @return true if `l` is a list, otherwise false
util.is_list = (l) ->
	type = type
	if (type l) != 'table'
		return false
	maxk = -1
	for k,_ in pairs l
		if (type k) != 'number'
			return false
		maxk = k if maxk < k
	maxk == #l

---
-- @brief Creates a coroutine which yields the characters of a string
-- @param s The string from which to yield
-- @return A coroutine which yields strings of length 1 which represent the individual characters of `s`
util.chars = (s) -> wrap -> yield s\sub i, i for i=1,len s

---
-- @brief Returns a string representing the character at a given index
-- @param i The index to obtain
-- @param s The string from which to extract
-- @return A string which contains just the `i`-th character of `s`
util.char_at = (i, s) -> s\sub i, i

---
-- @brief Converts a boolean value to an integer
-- @param b The value to check
-- @return true if `b` is considered true otherwise 0
util.bool_to_int = (b) ->
	return 1 if b
	0

---
-- @brief Computes the argmin of a function over a set of values
-- @param f The function to compute
-- @param vs a list of values to test
-- @return The argmin of `f` over `vs`, that is the _v_ in `vs` which minimises `f(v)`, also returns the value of `f(v)`
util.argmin = (f, vs) ->
	am = nil
	m = maxinteger
	for v in *vs
		t = f v
		if t < m
			m = t
			am = v
	am, m

---
-- @brief Computes the argmax of a function over a set of values
-- @param f The function to compute
-- @param vs a list of values to test
-- @return The argmax of `f` over `vs`, that is the _v_ in `vs` which maximises `f(v)`, also returns the value of `f(v)`
util.argmax = (f, vs) ->
	am = nil
	m = mininteger
	for v in *vs
		t = f v
		if m < t
			m = t
			am = v
	am, m

---
-- @brief Returns whether a value is an element of a list
-- @param v A value to check membership
-- @param vs A list of values to search
-- @return true if `v` is a value if `vs`, otherwise false
util.elem = (v, vs) ->
	for _,v2 in pairs vs
		if v == v2
			return true
	false

---
-- @brief Computes the result of extending a given list by other lists (pure)
-- @param xs A list to extend
-- @param ... Further lists
-- @return A list which is the concatenation of the lists passed, in the order passed.
util.extend = (xs, ...) ->
	zs = [ x for x in *xs ]
	for ys in *{...}
		insert zs, y for y in *ys
	zs

---
-- @brief Sort a list and then return it (impure)
-- @param t A list to sort
-- @param ... Further parameters to `table.sort`
-- @return `t`, having been sorted in place
util.sorted = (t, ...) ->
	sort t, ...
	t

---
-- @brief Recursively computes the equality of two values
-- @param a A value to check
-- @param b A value to check
-- @return true if `a` and `b` are equal, otherwise false
eq = (a,b) ->
	ta = type a
	tb = type b
	if ta != tb
		return false

	if ta != 'table' and tb != 'table'
		return a == b

	mt = getmetatable a
	if mt and mt.__eq
		return a == b

	for k1,v1 in pairs a
		v2 = b[k1]
		if v2 == nil or not eq v1,v2
			return false

	for k2,v2 in pairs b
		v1 = a[k2]
		if v1 == nil or not eq v1,v2
			return false
	true
util.eq = eq

---
-- @brief Returns whether a value is not nil
-- @param v Value to check
-- @return true if `v` is not nil otherwise false
util.non_nil = (v) -> v != nil

---
-- @brief Wrap a given function so that it is only run on a particular typesetting iteration
-- @param f The function to wrap
-- @return A function which takes a number _n_ and the parameters to _f_ which when called will only execute _f_ if the current typesetting loop iteration is equal to _n_
util.on_iter_wrap = (f) -> (n, ...) ->
	if 'number' != type n
		n = tonumber eval_string n
	if iter_num! == n
		f ...

class util.StringBuilder
	new: (@content={}) => -- @content is has type T where T <: primitive | [T] | {__tostring: => str}
	get_contents: => @content
	extend: (cs, d) =>
		if d
			return if #cs == 0
			insert @content, cs[1]
			for i=2,#cs
				insert @content, d
				insert @content, cs[i]
		else
			insert @content, c for c in *cs
	__concat: (s) =>
		insert @content, s
		@
	__call: =>
		flattened = {}
		flatten = (o) ->
			if 'table' == type o
				mt = getmetatable o
				if mt and mt.__tostring
					s = tostring o
					insert flattened, s unless s == ''
				else
					flatten e for e in *o
			else
				insert flattened, o if o != nil and o != ''
		flatten @content
		concat flattened

class util.Proxy
	new: (@_getters={}, @_setters={}) =>
	__get: (k) =>
		if getter = @getter k
			getter @, k
	getter: (k) => @_getters[k]
	__set: (k,v) =>
		if setter = @setter k
			setter @, k, v
		else
			error "Could not set proxy key '#{k}': unknown key"
	setter: (k) => @_setters[k]
meta_wrap util.Proxy

util
