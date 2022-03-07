---
-- @file std.ast
-- @brief Provides an interface for constructing Emblem document nodes
-- @author Edward Jones
-- @date 2021-09-17

import wrap_indices from require 'std.base'
import node_types from require 'std.constants'
import EphemeronTable from require 'std.data'
import show from require 'std.show'
import is_list, StringBuilder from require 'std.util'
import wrap, yield from coroutine
import insert from table

import WORD, CALL, CONTENT from node_types

import __em from _G
import
	__append_arg
	__append_child
	__copy
	__get_arg
	__get_attr
	__get_child
	__get_content_type
	__get_flags
	__get_last_eval
	__get_loc
	__get_name
	__get_num_children
	__get_parent
	__get_raw_word
	__get_result
	__get_sanitised_word
	__get_style
	__new_call
	__new_content
	__new_word
	__set_attr
	__set_flags
	from __em.__node
import __unpack_loc from __em

funcs = {
	'__append_arg',
	'__append_child',
	'__copy',
	'__get_arg',
	'__get_attr'
	'__get_child',
	'__get_content_type',
	'__get_flags'
	'__get_last_eval'
	'__get_loc'
	'__get_name'
	'__get_num_children',
	'__get_parent'
	'__get_raw_word'
	'__get_result',
	'__get_sanitised_word'
	'__get_style'
	'__new_call'
	'__new_content'
	'__new_word'
	'__set_attr'
	'__set_flags',
}
impld = 0
for func in *funcs
	print func, __em.__node[func]
	impld += 1 if __em.__node[func] != nil
print "Implemented interface functions: #{impld}/#{#funcs}"
print "__unpack_loc is undefined!" unless __em.__unpack_loc

nodes = EphemeronTable!

class Node
	new: (@_n) =>
		nodes[_n] = @
		@_loc = nil
	flag: (f) => 0 != f & __get_flags @_n
	set_flag: (f) => __set_flags @_n, f | __get_flags @_n
	flags: => __get_flags @_n
	set_flags: (f) => __set_flags @_n, f
	last_eval: => __get_last_eval @_n
	name: =>
		@_name = __get_name @_n unless @_name
		@_name
	style: => __get_style @_n
	parent: => nodes[__get_parent @_n]
	loc: =>
		@_loc = __get_loc @_n unless @_loc
		@_loc
	type: => __get_content_type @_n
	copy: => __copy @_n

	__tostring: => @show!
	show: => @repr!!
	repr: (sb=StringBuilder!) => sb .. "{Node #{@_n}}"

class AttrTable
	new: (@_n, attrs) =>
		wrap_indices @ -- TODO: Check memory leak here
		@[k] = v for k,v in ipairs attrs
	__get: (k) => __get_attr @_n, k
	__set: (k, v) => __set_attr @_n, k, v

class Location
	new: (@_loc) =>
		@_unpacked = nil
		wrap_indices @
	unpack: =>
		@_unpacked = __unpack_loc @_loc
		@_unpacked
	__get: (k) => @unpack![k]
	__set: => error "Location fields are read-only"

class Content
	new: (children={}) =>
		super __new_content!
		@append_child child for child in *children
	append_child: (c) => __append_child @_n, c._n
	len: => __get_num_children @_n
	iter: => wrap -> yield __get_child @_n, i for i = 1,@len!
	copy: =>
		ret = Content!
		ret\append_child c for c in @iter!

class Call
	new: (name, attrs={}) =>
		super __new_call name
		@attrs = AttrTable @_n, attrs
		with getmetatable @
			.__get = @attrs
			.__set = @attrs
		wrap_indices @
	result: => nodes[__get_result @_n]
	__tostring: => super!

class Word
	new: (word) => super __new_word word
	raw: => __get_raw_word @_n
	sanitised: => __get_sanitised_word @_n
	__tostring: => super!
	repr: (sb=StringBuilder!) => sb .. @raw!

class Node
	new: (@type, @flags=0) =>
	__tostring: => show @

sanitise_concat_input = (x) ->
	return {} if x == nil
	return {x} if ('table' != type x) or x.type == WORD or x.type == CALL
	return x.content if x.type == CONTENT
	error "Unrecognised concatenation input: #{show x}"

local Word
sanitise_content_item = (x) ->
	return Word x if 'table' != type x
	x

local Content
concat_ast_nodes = (as, bs) ->
	as2 = sanitise_concat_input as
	bs2 = sanitise_concat_input bs
	newlist = [ sanitise_content_item a for a in *as2 ]
	insert newlist, sanitise_content_item b for b in *bs2
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
