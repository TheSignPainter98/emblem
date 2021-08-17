import em, eval_string, get_var, set_var from require 'std.base'
import SyncMap from require 'std.events'
import log_warn_here from require 'std.log'

references = SyncMap!

CURR_LABEL_VAR_NAME = 'curr_label'
INITIAL_LABEL_VALUE = 'asdf'

set_label = (c) ->
	c = eval_string c
	set_var CURR_LABEL_VAR_NAME, c
get_label = -> get_var CURR_LABEL_VAR_NAME

set_label INITIAL_LABEL_VALUE

em.anchor = (key) ->
	key = eval_string key
	if references\has key
		log_warn_here "Duplicate anchor '#{key}'"
	references\add key, get_label!

em.ref = (key) ->
	key = eval_string key
	if not references\has key
		log_warn_here "Unknown anchor '#{key}'"
	references\get key

{ :get_label, :set_label }
