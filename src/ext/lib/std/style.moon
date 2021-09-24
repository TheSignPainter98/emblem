---
-- @file std.style
-- @brief Provides wrappers for basic styling directives
-- @author Edward Jones
-- @date 2021-09-17
import mkcall from require 'std.ast'

styles = { 'it', 'bf', 'sc', 'af', 'tt' }
stylers = { s, mkcall s for s in *styles }

{ :stylers }
