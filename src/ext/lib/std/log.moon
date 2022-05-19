---
-- @file std.log
-- @brief Provides logging functions
-- @author Edward Jones
-- @date 2021-09-17

import Directive, em, em_loc, eval_string, iter_num from require 'std.base'
import do_nothing, id from require 'std.func'
import on_iter_wrap from require 'std.util'
import format from string
import concat from table

log = {}

import __em from _G
for log_func in *{ 'log_err', 'log_err_at', 'log_warn', 'log_warn_at', 'log_info', 'log_debug' }
	handle_log_args = (...) -> concat [ eval_string e, true for e in *{...} ], ' '

	-- Handle exiting on error-logs
	afterop = do_nothing
	if log_func\match 'err'
		afterop = -> error 'Fatal error'

	if log_func\match '_at$'
		log_func_here_name = log_func\gsub '_at$', '_here'
		log[log_func] = (loc, ...) ->
			__em['__' .. log_func] loc, handle_log_args ...
			afterop!
		log[log_func_here_name] = (...) ->
			log[log_func] em_loc!, ...
			afterop!
		log[log_func .. '_on'] = on_iter_wrap log[log_func_here_name]
	else
		log[log_func] = (...) ->
			__em['__' .. log_func] handle_log_args ...
			afterop!
		log[log_func .. '_on'] = on_iter_wrap log[log_func]

em.error = Directive 0, -1, "Exit with an error", log.log_err_here
em.warn = Directive 0, -1, "Log a warning", log.log_warn_here
em.error_on = Directive 1, -1, "Exit with an error, but only on a given typesetting iteration", log.log_err_at_on
em.warn_on = Directive 1, -1, "Log a warning but only on a given typesetting iteration", log.log_warn_at_on

log
