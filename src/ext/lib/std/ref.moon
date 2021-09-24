---
-- @file std.ref
-- @brief Implements the label-anchor-reference system for cross-referencing within documents
-- @author Edward Jones
-- @date 2021-09-17

import Directive, em, eval_string, get_var, set_var from require 'std.base'
import SyncMap from require 'std.events'
import log_warn_here from require 'std.log'

references = SyncMap!

CURR_LABEL_VAR_NAME = 'curr_label'
INITIAL_LABEL_VALUE = '??'

---
-- @brief Set the current label
-- @param c The new label
set_label = (c) ->
	c = eval_string c
	set_var CURR_LABEL_VAR_NAME, c

---
-- @brief Get the label in the current context
-- @return The value of the label in the current context
get_label = -> get_var CURR_LABEL_VAR_NAME

set_label INITIAL_LABEL_VALUE

em.anchor = Directive 1, 0, "Set down an anchor with the given key", (key) ->
	key = eval_string key
	if references\has key
		log_warn_here "Duplicate anchor '#{key}'"
	references\add key, get_label!

em.ref = Directive 1, 0, "Reference an anchor with a given key", (key) ->
	key = eval_string key
	if not references\has key
		log_warn_here "Unknown anchor '#{key}'"
	references\get key

{ :get_label, :set_label }
