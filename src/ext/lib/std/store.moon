---
-- @file std.store
-- @brief Allows values to be stored between executions of Emblem
-- @author Edward Jones
-- @date 2021-09-17

import dump, load from require 'lyaml'
import Component from require 'std.events'
import log_err, log_warn from require 'std.log'
import Directive, em, wrap_indices from require 'std.base'

local open
if not io.module_unavailable
	import open from io

EM_STORE_FILE_DEFAULT = '.em-store.yml'

class Store extends Component
	new: (@store_loc=EM_STORE_FILE_DEFAULT) =>
		super!
		@curr_store = nil
		wrap_indices @
	ensure_has_store: =>
		if not open
			log_warn "Store unavailable due to sandbox level"
			rawset @, 'curr_store', {}
		elseif not rawget @, 'curr_store'
			f = open (rawget @, 'store_loc'), 'r'
			if not f
				rawset @, 'curr_store', {}
			else
				local lines
				with f
					lines = \read '*all'
					\close!
				curr_store = load lines
				rawset @, 'curr_store', curr_store or {}
	__get: (k, d) =>
		(rawget (getmetatable @), 'ensure_has_store') @
		(rawget (rawget @, 'curr_store'), k) or d
	__set: (k, v) =>
		(rawget (getmetatable @), 'ensure_has_store') @
		(rawget @, 'curr_store')[k] = v
	on_end: =>
		if @curr_store and open
			curr_store_rep = dump {@curr_store}
			f = open @store_loc, 'w'
			if not f
				log_err "Failed to open store file #{@store_loc}"
			with f
				\write curr_store_rep
				\close!

store = Store!

curr_version_num = nil
em.curr_version = Directive 0, 0, "Return the number of times this document has been compiled", ->
	if not curr_version_num
		curr_version_num = 1 + (store['comp-num'] or 0)
		store['comp-num'] = curr_version_num
	curr_version_num

{ :Store, :store }
