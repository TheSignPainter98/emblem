import node_types from require 'std.base'
import show from require 'std.show'
import is_list from require 'std.util'
import insert from table

import WORD, CALL, CONTENT from node_types

class Node
	new: (@type, @flags=0) =>
	__tostring: => show @

sanitise_concat_input = (x) ->
	return {} if x == nil
	return {x} if ('table' != type x) or x.type == WORD or x.type == CALL
	return x.content if x.type == CONTENT
	error "Unrecognised concatenation input: #{show x}"

local Content
concat_ast_nodes = (as, bs) ->
	as2 = sanitise_concat_input as
	bs2 = sanitise_concat_input bs
	newlist = [ a for a in *as2 ]
	insert newlist, b for b in *bs2
	flags = nil
	if as.type == bs.type and bs.type == CONTENT
		flags = as.flags
		if flags == nil
			flags = bs.flags
		elseif bs.flags != nil
			flags |= bs.flags
	Content newlist, flags

class Word extends Node
	new: (@word, ...) => super WORD, ...
	__concat: concat_ast_nodes

class Content extends Node
	new: (@content, ...) => super CONTENT, ...
	__concat: concat_ast_nodes

class Call extends Node
	new: (@name, @args, ...) =>
		super CALL, ...
		@args = {@args} if not is_list @args
	__concat: concat_ast_nodes
	__shl: (c, a) ->
		if 'table' != type c or c.type != CALL
			error "Left operand to an argument-append must be a call, instead got #{show c}"
		newargs = [ arg for arg in *c.args ]
		insert newargs, a
		Call c.name, newargs, c.flags

mkcall = (name) -> (args) -> Call name, args

{ :Call, :Content, :Word, :mkcall }
