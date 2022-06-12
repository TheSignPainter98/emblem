---
-- @file std.ast
-- @brief Provides an interface for constructing Emblem document nodes
-- @author Edward Jones
-- @date 2021-09-17

local Node, Word, Content, mktext

import em_loc, is_instance, unpack_loc, meta_wrap from require 'std.base'
import node_types from require 'std.constants'
import EphemeronTable, Set, WeakValueTable from require 'std.data'
import log_err_at, log_warn_at from require 'std.log'
import show from require 'std.show'
import unpack from table
import is_list, StringBuilder, Proxy, unite from require 'std.util'
import wrap, yield from coroutine
import concat, insert from table

import WORD, CALL, CONTENT from node_types

import __em from _G
import __get_loc_id from __em
import
	__append_arg
	__append_child
	__copy
	__eval
	__get_arg
	__get_attr
	__get_child
	__get_content_type
	__get_flags
	__get_last_eval
	__get_loc
	__get_name
	__get_num_args
	__get_num_children
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

import css from require 'std.constants'
import PSEUDO_ELEMENT_FIRST_LINE, PSEUDO_ELEMENT_FIRST_LETTER, PSEUDO_ELEMENT_BEFORE, PSEUDO_ELEMENT_AFTER from css

{__get_id: __get_node_id } = __em.__node

---
-- @brief A cache for core pointers and their representations, maps unique IDs to Lua-objects, which are created as necessary. As pointers are stored weakly, there is no guarantee getting two values from the map will return the same object, if that previously-gotten object has been garbage-collected.
class CorePointerMap
	_id_ptrs: WeakValueTable!
	_ptr_vals: EphemeronTable!

	get_ud_id: -> error "Not implemented gui", 2
	mk_obj: -> error "Not implemented mo", 2

	__set: (ptr, obj=@mk_obj ptr) =>
		@_ptr_vals[ptr] = obj
		@_id_ptrs[@get_ud_id ptr] = ptr
	__get: (k) =>
		id = switch type k
			when 'number'
				k
			when 'userdata'
				@get_ud_id k
			when 'table'
				k.id
			when 'nil'
				return nil
			else
				error "Index to #{@@__name} must be a number, userdata, table or nil, got a #{type k}: #{k}"
		@_ptr_vals[@_id_ptrs[id]]
	__tostring: => @@__name
meta_wrap CorePointerMap

---
-- @brief Wrapper for a core location pointer
class Location
	new: (@_loc) =>
	unpack: =>
		rawset @, '_unpacked', unpack_loc @_loc unless rawget @, '_unpacked'
		@_unpacked
	id: => __get_loc_id @_loc
	__get: (k) => @unpack![k]
	__set: (k,v) => error "Location fields are read-only, #{k}, #{v}", 2
	__tostring: => show @unpack!
meta_wrap Location

---
-- @brief Cache for location objects
class LocMap extends CorePointerMap
	__tostring: => super!
	get_ud_id: (u) => __get_loc_id u
	mk_obj: (p) => Location p

---
-- @brief Location cache
__em.locs = LocMap!

class NodeProxy
	new: (@_n, @_get_proxies={}, _cache_fields={}, @_set_proxies={}) =>
		@_cache = {}
		@_cache_fields = Set _cache_fields
	__get: (k) =>
		if v = @_cache[k]
			return v
		if f = @_get_proxies[k]
			v = f @_n
			@_cache[k] = v if @_cache_fields[k]
			v
		else
			error "Unknown proxy field '#{k}', expected one of: #{concat [ k for k,_ in pairs @_get_proxies ], ', '}", 2
	__set: (k, v) =>
		@_cache[k] = nil
		if f = @_set_proxies[k]
			f @_n, v
		else
			rawset @, k, v
	__pairs: => wrap -> yield k, f @_n for k,f in pairs @_get_proxies
meta_wrap NodeProxy

import
	__get_align_content
	__get_align_items
	__get_align_self
	__get_background
	__get_background_attachment
	__get_background_position
	__get_border_bottom
	__get_border_bottom_style
	__get_border_width_bottom
	__get_border_collapse
	__get_border_left
	__get_border_left_style
	__get_border_width_left
	__get_border_right
	__get_border_right_style
	__get_border_width_right
	__get_border_spacing
	__get_border_top
	__get_border_top_style
	__get_border_width_top
	__get_bottom
	__get_box_sizing
	__get_break_after
	__get_break_before
	__get_break_inside
	__get_caption_side
	__get_clear
	__get_clip
	__get_colour
	__get_column_count
	__get_column_fill
	__get_column_gap
	__get_column_rule
	__get_column_rule_style
	__get_column_rule_width
	__get_column_span
	__get_column_width
	__get_direction
	__get_display
	__get_display_static
	__get_empty_cells
	__get_flex_basis
	__get_flex_direction
	__get_flex_grow
	__get_flex_shrink
	__get_flex_wrap
	__get_float
	__get_font_family
	__get_font_size
	__get_font_style
	__get_font_variant
	__get_font_weight
	__get_height
	__get_justify_content
	__get_left
	__get_letter_spacing
	__get_line_height
	__get_list_style_position
	__get_list_style_type
	__get_margin_bottom
	__get_margin_left
	__get_margin_right
	__get_margin_top
	__get_max_height
	__get_max_width
	__get_min_height
	__get_min_width
	__get_opacity
	__get_order
	__get_orphans
	__get_outline
	__get_outline_style
	__get_outline_width
	__get_overflow_x
	__get_overflow_y
	__get_padding_bottom
	__get_padding_left
	__get_padding_right
	__get_padding_top
	__get_page_break_after
	__get_page_break_before
	__get_page_break_inside
	__get_position
	__get_right
	__get_table_layout
	__get_text_align
	__get_text_decoration
	__get_text_indent
	__get_text_transform
	__get_top
	__get_unicode_bidi
	__get_vertical_align
	__get_visibility
	__get_white_space
	__get_widows
	__get_width
from __em.__css
import
	__get_word_spacing
	__get_writing_mode
	__get_z_index
from __em.__css

class Style extends Proxy
	new: (@_n, @_pseudo=false) =>
		super @style_map
		@_subdomains = EphemeronTable!
	__get: (k) =>
		if subdomain = @pseudo_elems[k]
			if @_pseudo
				error "Cannot get pseudo-element of a pseudo-element: tried to extract a #{k} pseudo-elem from a #{@_pseudo} pseudo-elem"
			s = Style @_n, subdomain
			@_subdomains[s] = true
			s
		if f = @getter k
			f @_n, @_pseudo or nil
	unpack: => __get_style @_n, @_pseudo or nil
	pseudo_elems:
		first_letter: PSEUDO_ELEMENT_FIRST_LETTER
		first_line: PSEUDO_ELEMENT_FIRST_LINE
		before: PSEUDO_ELEMENT_BEFORE
		after: PSEUDO_ELEMENT_AFTER
	style_map: {
		align_content: __get_align_content
		align_items: __get_align_items
		align_self: __get_align_self
		background: __get_background
		background_attachment: __get_background_attachment
		background_position: __get_background_position
		border_bottom: __get_border_bottom
		border_bottom_style: __get_border_bottom_style
		border_bottom_width: __get_border_width_bottom
		border_collapse: __get_border_collapse
		border_left: __get_border_left
		border_left_style: __get_border_left_style
		border_left_width: __get_border_width_left
		border_right: __get_border_right
		border_right_style: __get_border_right_style
		border_right_width: __get_border_width_right
		border_spacing: __get_border_spacing
		border_top: __get_border_top
		border_top_style: __get_border_top_style
		border_top_width: __get_border_width_top
		bottom: __get_bottom
		box_sizing: __get_box_sizing
		break_after: __get_break_after
		break_before: __get_break_before
		break_inside: __get_break_inside
		caption_side: __get_caption_side
		clear: __get_clear
		clip: __get_clip
		color: __get_colour
		colour: __get_colour
		column_count: __get_column_count
		column_fill: __get_column_fill
		column_gap: __get_column_gap
		column_rule: __get_column_rule
		column_rule_style: __get_column_rule_style
		column_rule_width: __get_column_rule_width
		column_span: __get_column_span
		column_width: __get_column_width
		direction: __get_direction
		display: __get_display
		display_static: __get_display_static
		empty_cells: __get_empty_cells
		flex_basis: __get_flex_basis
		flex_direction: __get_flex_direction
		flex_grow: __get_flex_grow
		flex_shrink: __get_flex_shrink
		flex_wrap: __get_flex_wrap
		float: __get_float
		font_family: __get_font_family
		font_size: __get_font_size
		font_style: __get_font_style
		font_variant: __get_font_variant
		font_weight: __get_font_weight
		height: __get_height
		justify_content: __get_justify_content
		left: __get_left
		letter_spacing: __get_letter_spacing
		line_height: __get_line_height
		list_style_position: __get_list_style_position
		list_style_type: __get_list_style_type
		margin_bottom: __get_margin_bottom
		margin_left: __get_margin_left
		margin_right: __get_margin_right
		margin_top: __get_margin_top
		max_height: __get_max_height
		max_width: __get_max_width
		min_height: __get_min_height
		min_width: __get_min_width
		opacity: __get_opacity
		order: __get_order
		orphans: __get_orphans
		outline: __get_outline
		outline_style: __get_outline_style
		outline_width: __get_outline_width
		overflow_x: __get_overflow_x
		overflow_y: __get_overflow_y
		padding_bottom: __get_padding_bottom
		padding_left: __get_padding_left
		padding_right: __get_padding_right
		padding_top: __get_padding_top
		page_break_after: __get_page_break_after
		page_break_before: __get_page_break_before
		page_break_inside: __get_page_break_inside
		position: __get_position
		right: __get_right
		table_layout: __get_table_layout
		text_align: __get_text_align
		text_decoration: __get_text_decoration
		text_indent: __get_text_indent
		text_transform: __get_text_transform
		top: __get_top
		unicode_bidi: __get_unicode_bidi
		vertical_align: __get_vertical_align
		visibility: __get_visibility
		white_space: __get_white_space
		widows: __get_widows
		width: __get_width
		word_spacing: __get_word_spacing
		writing_mode: __get_writing_mode
		z_index: __get_z_index
	}

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
class Node extends NodeProxy
	new: (@_n, get_fields, cache_fields, set_fields) =>
		super @_n, (unite @node_get_fields, get_fields), (unite @node_cache_fields, cache_fields), unite @node_set_fields, set_fields
		unless 'userdata' == type _n
			error "Node requires a core pointer, got a #{type _n}: #{_n}", 2
		__em.nodes[_n] = @ unless __em.nodes[_n]
		@style = Style _n
	node_get_fields: {
		flags: __get_flags
		id: __get_node_id
		last_eval: __get_last_eval
		loc: __get_loc
		name: __get_name
		type: __get_content_type
	}
	node_set_fields: {
		flags: __set_flags
	}
	node_cache_fields: {
		'id'
		'loc'
		'name'
		'type'
	}
	eval: => __eval @_n
	flag: (f) => 0 != f & __get_flags @_n
	set_flag: (f) => __set_flags @_n, f | __get_flags @_n
	set_flags: (f) => __set_flags @_n, f
	copy: => __copy @_n
	error: (...) => log_err_at (__get_loc @_n), ...
	warn: (...) => log_warn_at (__get_loc @_n), ...

	__tostring: => @node_string true
	eval_string: (pretty=false) =>
		@eval!
		@node_string pretty
	node_string: (pretty=false) => (@_node_string StringBuilder!, pretty)!
	show: => @repr!!
	repr: (sb=StringBuilder!) => sb .. "{Node #{@_n} (type=#{__get_content_type @_n})}"
	__call: => @eval!

	sanitise_concat_arg: (a) =>
		switch type a
			when 'table'
				if is_instance Node, a
					return a
				else if is_list a
					return Content { @sanitise_concat_arg a2 for a2 in *a }
		as = { mktext a }
		return as[1] if #as == 1
		Content as
	__concat: (a,b) ->
		a = Node\sanitise_concat_arg a
		b = Node\sanitise_concat_arg b
		if is_instance Content, a
			unless is_instance Content, b
				a\append_child b
				return b
		Content {a, b}

---
-- @brief Proxy for call attributes
class AttrTable
	new: (@_n, attrs) =>
		__set_attr @_n, k, v for k,v in ipairs attrs
	__get: (k) => __get_attr @_n, k
	__set: (k, v) => __set_attr @_n, k, v
meta_wrap AttrTable

---
-- @brief Wrapper for content nodes (those which can have other nodes beneath them without affecting styling or calling extension funcionality
class Content extends Node
	new: (children={}) =>
		if 'userdata' == type children
			super children
		else
			super __new_content em_loc!
			@append_child child for child in *children
	append_child: (c) => __append_child @_n, c._n
	__add: (c) =>
		@append_child c
		@
	__len: => __get_num_children @_n
	__pairs: =>
		i,n = 0,#@
		->
			i += 1
			i, __em.nodes[__get_child @_n, i] if i <= n
	__getidx: (i) =>
		return nil unless 1 <= i and i <= @__len!
		__em.nodes[__get_child @_n, i]
	iter: => wrap -> yield i, __em.nodes[__get_child @_n, i] for i = 1,#@
	__tostring: => super!
	__call: (...) => super ...
	_node_string: (sb, pretty) =>
		first = true
		for _,c in @iter!
			unless first
				sb .. ' '
			else
				first = false
			c\_node_string sb, pretty
		sb
	repr: (sb=StringBuilder!) =>
		sb .. "{Content ["
		for i = 1, @__len!
			sb .. ', ' unless i == 1
			sb .. (__get_child @_n, i)\repr sb
		sb .. ']}'

---
-- @brief Wrapper for call nodes, which can affect styling and which can cause extension functions to be called
class Call extends Node
	new: (name, args={}, attrs={}) =>
		args = { mktext args } if 'string' == type args
		switch type name
			when 'userdata'
				super name, @call_get_fields
			when 'table'
				error "Trying to construct a node out of a table doesn't work!"
			else
				super (__new_call name, args, em_loc!), @call_get_fields
		@attrs = AttrTable @_n, attrs
	call_get_fields: {
		args: (n) -> [ __em.nodes[__get_arg n, i] for i = 1, __get_num_args n ]
		result: (n) ->
			if r = __get_result n
				__em.nodes[r]
	}
	__getidx: (i) => __em.nodes[__get_arg @_n, i]
	__tostring: => super!
	__call: (...) => super ...
	__len: => __get_num_args @_n
	_node_string: (sb, pretty) =>
		if r = __get_result @_n
			__em.nodes[r]\_node_string sb, pretty
		else
			sb
	repr: (sb=StringBuilder!) =>
		sb .. "{Call:#{__get_name @_n}("
		sb .. ')->{'
		if r = __get_result @_n
			sb .. __em.nodes[r]\repr sb
		sb .. '}}'

---
-- @brief Wrapper for word nodes, which represents single parts of text.
class Word extends Node
	new: (word) =>
		switch type word
			when 'userdata'
				super word, @word_get_fields, @word_cache_fields
			else
				super (__new_word word, em_loc!), @word_get_fields, @word_cache_fields
	word_get_fields: {
		raw: __get_raw_word
		pretty: __get_sanitised_word
	}
	word_cache_fields: {
		'raw'
		'pretty'
	}
	repr: (sb=StringBuilder!) => sb .. "{Word<#{__get_raw_word @_n}>}"
	__tostring: => super!
	__call: (...) => super ...
	_node_string: (sb, pretty) => sb .. (pretty and @pretty or @raw)

with __em.nodes.raw_node_constructors
	[WORD] = Word
	[CONTENT] = Content
	[CALL] = Call

---
-- @brief Make word nodes
-- @param words The text to split by whitespace to form words
-- @return Returns the word nodes, each as a return value.
mktext = (words) -> unpack [ Word w for w in (tostring words)\gmatch '%s*([^%s]+)' ]

---
-- @brief Make a function which constructs a call to a given directive
-- @param Name of the directive to call
-- @return A function which takes arguments and returns a call upon those arguments
mkcall = (name) -> (args) -> Call name, args

{ :Call, :Content, :Word, :mkcall, :mktext }
