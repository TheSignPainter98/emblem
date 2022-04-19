---
-- @file std.store
-- @brief Allows values to be stored between executions of Emblem
-- @author Edward Jones
-- @date 2021-09-17

import dump, load from require 'lyaml'
import Component from require 'std.events'
import log_err, log_warn from require 'std.log'
import Directive, em, meta_wrap from require 'std.base'

local open
if not io.module_unavailable
	import open from io

EM_STORE_FILE_DEFAULT = '.em-store.yml'

---
-- @brief Represents store of values to transfer between separate runs of the program
class Store extends Component
	new: (@store_loc=EM_STORE_FILE_DEFAULT) =>
		super!
		@curr_store = false
	ensure_has_store: =>
		if not open
			log_warn "Store unavailable due to sandbox level"
			@curr_store = {}
		elseif not @curr_store
			f = open @store_loc, 'r'
			if not f
				@curr_store = {}
			else
				local lines
				with f
					lines = \read '*all'
					\close!
				@curr_store = (load lines) or {}
	__get: (k, d) =>
		@ensure_has_store!
		@curr_store[k] or d
	__set: (k, v) =>
		@ensure_has_store!
		@curr_store[k] = v
	on_end: =>
		if @curr_store and open
			curr_store_rep = dump {@curr_store}
			f = open @store_loc, 'w'
			if not f
				log_err "Failed to open store file #{@store_loc}"
			with f
				\write curr_store_rep
				\close!
meta_wrap Store

---
-- @brief The current score
store = Store!

curr_version_num = nil
em.curr_version = Directive 0, 0, "Return the number of times this document has been compiled", ->
	if not curr_version_num
		curr_version_num = 1 + (store['comp-num'] or 0)
		store['comp-num'] = curr_version_num
	curr_version_num

{ :Store, :store }
