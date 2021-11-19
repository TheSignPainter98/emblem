---
-- @file std.ast
-- @brief Provides an interface for constructing Emblem document nodes
-- @author Edward Jones
-- @date 2021-09-17

import node_types from require 'std.constants'
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
	if ('table' == type as) and ('table' == type bs) and as.type == bs.type and bs.type == CONTENT
		flags = as.flags
		if flags == nil
			flags = bs.flags
		elseif bs.flags != nil
			flags |= bs.flags
	Content newlist, flags

---
-- @brief Represents a word node
class Word extends Node
	new: (@word, ...) => super WORD, ...
	__tostring: => show @
	__concat: concat_ast_nodes

---
-- @brief Represents a content node (which has no content itself but rather stores other nodes beneath it)
class Content extends Node
	new: (@content, ...) => super CONTENT, ...
	__tostring: => show @
	__concat: concat_ast_nodes

---
-- @brief Represents a call to a directive
class Call extends Node
	new: (@name, @args, ...) =>
		super CALL, ...
		@args = {@args} if not is_list @args
	__tostring: => show @
	__concat: concat_ast_nodes
	__mul: (c, a) ->
		if 'table' != type c or c.type != CALL
			error "Left operand to an argument-append must be a call, instead got #{show c}"
		newargs = [ arg for arg in *c.args ]
		insert newargs, a
		Call c.name, newargs, c.flags

---
-- @brief Make a function which constructs a call to a given directive
-- @param Name of the directive to call
-- @return A function which takes arguments and returns a call upon those arguments
mkcall = (name) -> (args) -> Call name, args

{ :Call, :Content, :Word, :mkcall }
