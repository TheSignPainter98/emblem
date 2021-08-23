import open from io
import dump, load from require 'lyaml'
import Component from require 'std.events'
import log_err from require 'std.log'
import em, wrap_index from require 'std.base'

EM_STORE_FILE_DEFAULT = '.em-store.yml'

class Store extends Component
	new: (@store_loc=EM_STORE_FILE_DEFAULT) =>
		super!
		wrap_index @
		@curr_store = nil
	ensure_has_store: =>
		if not rawget @, 'curr_store'
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
		if ret = (rawget @, 'curr_store')[k]
			ret
		else
			d
	__set: (k, v) =>
		(rawget (getmetatable @), 'ensure_has_store') @
		(rawget @, 'curr_store')[k] = v
	on_end: =>
		if @curr_store
			curr_store_rep = dump {@curr_store}
			f = open @store_loc, 'w'
			if not f
				log_err "Failed to open store file #{@store_loc}"
			with f
				\write curr_store_rep
				\close!

store = Store!

curr_version_num = nil
em.curr_version = ->
	if not curr_version_num
		curr_version_num = 1 + (store['comp-num'] or 0)
		store['comp-num'] = curr_version_num
	curr_version_num

{ :Store, :store }
