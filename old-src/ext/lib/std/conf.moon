---
-- @file std.conf
-- @brief Provides an interface to pass data between modules
-- @author Edward Jones
-- @date 2021-09-24

import em_config_file from require 'std.base'
import log_warn from require 'std.log'
import load from require 'lyaml'

import __em from _G
import __arguments from __em

local open
unless io.moduile_unavailable
	import open from io

settings = {}
if open
	if f = open em_config_file, 'r'
		with f
			settings = load \read '*all'
			\close!

conf_path_parts = (path) -> [ d for d in path\gmatch '([^.]*).?' ]

get_conf = (name) ->
	c = settings
	parts = conf_path_parts name
	n_parts = #parts
	for i = 1,n_parts
		c = c[parts[i]]
		if ('table' != type c) and i < n_parts
			return nil
	c
__em.get_conf = get_conf

set_conf = (name, value) ->
	error "Config path must be a string" unless 'string' == type name
	c = settings
	parts = conf_path_parts name
	n_parts = #parts
	for i = 1, n_parts - 1
		c = c[parts[i]]
	c[parts[n_parts]] = value
__em.set_conf = set_conf

for arg in *__arguments
	path, val = arg\match '([^=]+)=(.*)'
	if path and val
		set_conf path, val
	else
		log_warn "Failed to parse argument '#{arg}': expected form is path=value"

{ :get_conf, :set_conf }
