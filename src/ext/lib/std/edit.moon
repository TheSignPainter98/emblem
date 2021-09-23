import Directive, em from require 'std.base'
import min from math
import len from string
import concat, insert from table
import argmin, bool_to_int, char_at, chars, sorted from require 'std.util'

-- minmat is in row-major order

init_mat = (ul, vl) ->
	mat = { [ i for i=0,vl ] }
	insert mat, { i } for i=1,ul-1
	mat

show_mat = (u, v, ul, vl, minmat) ->
	ret = { 'X\t \t' .. concat [ c for c in chars v ], '\t' }
	insert ret, "#{char_at r - 1, u}\t" .. concat [ minmat[r][c] or '_' for c=1,vl ], '\t' for r=1,ul
	concat ret, '\n'

edit_distance = (u, v) ->
	ul = 1 + len u
	vl = 1 + len v
	minmat = init_mat ul, vl
	for i = 2, ul
		for j = 2, vl
			sub = minmat[i - 1][j - 1] + bool_to_int (char_at i - 1, u) != char_at j - 1, v
			ins = minmat[i - 1][j] + 1
			del = minmat[i][j - 1] + 1
			minmat[i][j] = min sub, ins, del
	minmat[ul][vl]

closest = (s, ts) ->
	f = (t) -> edit_distance s, t
	argmin f, ts

SUGGESTION_THRESHOLD = 2
unknown_x_msg = (x, v, vs) ->
	c,d = closest v, vs
	suggestion = '.'
	if d <= SUGGESTION_THRESHOLD
		suggestion = " perhaps you meant '#{c}'?"
	"Unknown #{x}, '#{v}'#{suggestion} Expected one of:" .. concat [ "\n\t#{d}" for d in *sorted vs ]

{ :edit_distance, :closest, :unknown_x_msg }
