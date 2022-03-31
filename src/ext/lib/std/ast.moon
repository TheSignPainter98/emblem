---
-- @file std.ast
-- @brief Provides an interface for constructing Emblem document nodes
-- @author Edward Jones
-- @date 2021-09-17

import wrap_indices from require 'std.base'
import node_types from require 'std.constants'
import EphemeronTable, WeakValueTable from require 'std.data'
import log_err_at, log_warn_at from require 'std.log'
import show from require 'std.show'
import is_list, StringBuilder from require 'std.util'
import wrap, yield from coroutine
import insert from table

import WORD, CALL, CONTENT from node_types

import __em from _G
import __get_loc_id from __em
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

{__get_id: __get_node_id } = __em.__node

---
-- @brief A cache for core pointers and their representations, maps unique IDs to Lua-objects, which are created as necessary. As pointers are stored weakly, there is no guarantee getting two values from the map will return the same object, if that previously-gotten object has been garbage-collected.
class CorePointerMap
	new: => wrap_indices @
	_id_ptrs: WeakValueTable!
	_ptr_vals: EphemeronTable!

	get_ud_id: -> error "Not implemented gui", 2
	mk_obj: -> error "Not implemented mo", 2

	__set: (ptr, obj=@mk_obj ptr) =>
		@_ptr_vals[ptr] = obj
		@_id_ptrs[obj\id!] = ptr
	__get: (k) =>
		id = switch type k
			when 'number'
				k
			when 'userdata'
				@get_ud_id k
			when 'table'
				k\id!
			else
				error "Index to #{@@__name} must be a number, userdata or a table"
		@_ptr_vals[@_id_ptrs[id]]
	__tostring: => @@__name

---
-- @brief Wrapper for a core location pointer
class Location
	new: (_loc) =>
		rawset @, '_loc', _loc
		wrap_indices @
	unpack: =>
		rawset @, '_unpacked', unpack_loc @_loc unless rawget @, '_unpacked'
		@_unpacked
	id: => __get_loc_id @_loc
	__get: (k) => @unpack![k]
	__set: (k,v) => error "Location fields are read-only, #{k}, #{v}", 2
	__tostring: => show @unpack!

---
-- @brief Cache for location objects
class LocMap extends CorePointerMap
	__tostring: => super!
	get_ud_id: (u) => __get_loc_id u
	mk_obj: (p) => Location p

---
-- @brief Location cache
__em.locs = LocMap!

---
-- @brief Cache for node objects
class NodeMap extends CorePointerMap
	__tostring: => super!
	get_ud_id: (u) => __get_node_id u
	raw_node_constructors: {}
	mk_obj: (p) => @raw_node_constructors[__get_content_type p] p

---
-- @brief Node cache
__em.nodes = NodeMap!

---
-- @brief Base class for wrappers for core node pointers
class Node
	new: (@_n, flags=0) =>
		__em.nodes[_n] = @
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
	loc: =>
		@_loc = __get_loc @_n unless @_loc
		@_loc
	type: => __get_content_type @_n
	copy: => __copy @_n
	error: (...) => log_err_at @loc!, ...
	warn: (...) => log_warn_at @loc!, ...

	__tostring: => @node_string true
	node_string: (pretty=false) => (@_node_string StringBuilder!, pretty)!
	show: => @repr!!
	repr: (sb=StringBuilder!) => sb .. "{Node #{@_n} (type=#{@type!})}"
	__call: => @eval!

---
-- @brief Proxy for call attributes
class AttrTable
	new: (_n, attrs) =>
		rawset @, '_n', _n
		__get_attr @_n, k, v for k,v in ipairs attrs
		wrap_indices @
	__get: (k) => __get_attr (rawget @, '_n'), k
	__set: (k, v) => __set_attr (rawget @, '_n'), k, v

---
-- @brief Wrapper for content nodes (those which can have other nodes beneath them without affecting styling or calling extension funcionality
class Content extends Node
	new: (children={}) =>
		if 'userdata' == type children
			super children
		else
			super __new_content em_loc!
			@append_child child for child in *children
		wrap_indices @
	append_child: (c) => __append_child @_n, c._n
	__add: (c) =>
		@append_child c
		@
	__len: => __get_num_children @_n
	__pairs: =>
		i,n = 0,#@
		->
			i += 1
			i, nodes[__get_child @_n, i] if i <= n
	__get: (i) => nodes[__get_child @_n, i]
	iter: => wrap -> yield __get_child @_n, i for i = 1,#@
	copy: =>
		ret = Content!
		ret\append_child c for c in @iter!
	__tostring: => super!

---
-- @brief Wrapper for call nodes, which can affect styling and which can cause extension functions to be called
class Call extends Node
	new: (name, args={}, attrs={}) =>
		switch type name
			when 'userdata'
				super name
			else
				super __new_call name, args, em_loc!
		with getmetatable @
			.__get = @attrs
			.__set = @attrs
		wrap_indices @
	arg: (i) => __get_arg @_n, i
	result: =>
		if r = __get_result @_n
			__em.nodes[r]
		else
			nil
	__tostring: => super!

---
-- @brief Wrapper for word nodes, which represents single parts of text.
class Word extends Node
	new: (word) =>
		switch type word
			when 'userdata'
				super word
			else
				super __new_word word, em_loc!
	raw: => __get_raw_word @_n
	sanitised: => __get_sanitised_word @_n
	repr: (sb=StringBuilder!) => sb .. @raw!
	__tostring: => super!

with __em.nodes.raw_node_constructors
	[WORD] = Word
	[CONTENT] = Content
	[CALL] = Call

---

---
-- @brief Make a function which constructs a call to a given directive
-- @param Name of the directive to call
-- @return A function which takes arguments and returns a call upon those arguments
mkcall = (name) -> (args) -> Call name, args

{ :Call, :Content, :Word, :mkcall }
