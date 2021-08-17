import em, em_loc, eval_string, _log_err, _log_err_at, _log_warn, _log_warn_at, _log_info, _log_debug, iter_num from require 'std.base'
import do_nothing, id from require 'std.func'
import on_iter_wrap from require 'std.util'
import format from string
import concat from table

log = {}

base = require 'std.base'
log_funcs = { 'log_err', 'log_err_at', 'log_warn', 'log_warn_at', 'log_info', 'log_debug' }
for log_func in *log_funcs
	handle_log_args = (...) -> concat [ eval_string e for e in *{...} ], ' '

	-- Handle exiting on error-logs
	afterop = do_nothing
	if log_func\match 'err'
		afterop = -> error 'Fatal error'

	if log_func\match '_at$'
		log_func_here_name = log_func\gsub '_at$', '_here'
		log[log_func .. '_loc'] = (loc, ...) ->
			base['_' .. log_func] loc, handle_log_args ...
			afterop!
		log[log_func_here_name] = (...) ->
			log[log_func .. '_loc'] em_loc!, ...
			afterop!
		log[log_func .. '_on'] = on_iter_wrap log[log_func_here_name]
	else
		log[log_func] = (...) ->
			base['_' .. log_func] handle_log_args ...
			afterop!
		log[log_func .. '_on'] = on_iter_wrap log[log_func]

em.error = log.log_err_here
em.warn = log.log_warn_here
em['error-on'] = log.log_err_at_on
em['warn-on'] = log.log_warn_at_on

log
