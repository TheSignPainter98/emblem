---
-- @file std.data
-- @brief Provides standard datatypes
-- @author Edward Jones
-- @date 2022-02-09

import meta_wrap from require 'std.base'

class EphemeronTable
	__mode: 'k'

class WeakValueTable
	__mode: 'v'

class WeakTable
	__mode: 'kv'

class Set
	new: (data) => @_data = { d, true for d in *data }
	__get: (k) => @_data[k]
	__set: (k, v) => @_data[k] = not v
meta_wrap Set

{ :EphemeronTable, :WeakValueTable, :WeakTable, :Set, }
