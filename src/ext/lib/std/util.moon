---
-- @file std.util
-- @brief Provides miscellaneous utility functions
-- @author Edward Jones
-- @date 2021-09-17

import eval_string, iter_num from require 'std.base'
import wrap, yield from coroutine
import maxinteger, mininteger from math
import len from string
import insert, sort from table

util = {}

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

util.chars = (s) -> wrap -> yield s\sub i, i for i=1,len s

util.char_at = (i, s) -> s\sub i, i

util.bool_to_int = (b) ->
	return 1 if b
	0

util.argmin = (f, vs) ->
	am = nil
	m = maxinteger
	for v in *vs
		t = f v
		if t < m
			m = t
			am = v
	am, m

util.argmax = (f, vs) ->
	am = nil
	m = mininteger
	for v in *vs
		t = f v
		if m < t
			m = t
			am = v
	am, m

util.elem = (v, vs) ->
	for _,v2 in pairs vs
		if v == v2
			return true
	false

util.extend = (xs, ...) ->
	zs = [ x for x in *xs ]
	for ys in *{...}
		insert zs, y for y in *ys
	zs

util.sorted = (t, ...) ->
	sort t, ...
	t

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

util.non_nil = (v) -> v != nil

util.on_iter_wrap = (f) ->
	(n, ...) ->
		if 'number' != type n
			n = tonumber eval_string n
		if iter_num! == n
			f ...

util
