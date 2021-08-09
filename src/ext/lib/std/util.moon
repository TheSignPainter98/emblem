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

util.elem = (v, vs) ->
	for _,v2 in pairs vs
		if v == v2
			return true
	false

util.extend = (xs, ys) ->
	zs = [ x for x in *xs ]
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

util
