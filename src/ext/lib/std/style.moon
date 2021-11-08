---
-- @file std.style
-- @brief Provides wrappers for basic styling directives
-- @author Edward Jones
-- @date 2021-09-17
import mkcall from require 'std.ast'
import stylesheet from require 'std.base'
import format from string
import concat from table

stylesheet 'std/style.scss'

styles = { 'it', 'bf', 'sc', 'df', 'af', 'tt' }
stylers = { s, mkcall s for s in *styles }

colour_to_hex = (col) -> concat [ format '%02x', col[k] for k in *{ 'r', 'g', 'b' } ]

{ :colour_to_hex, :stylers }
