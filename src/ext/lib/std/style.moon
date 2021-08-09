import mkcall from require 'std.ast'

styles = { 'it', 'bf', 'sc', 'af', 'tt' }
stylers = { s, mkcall s for s in *styles }

{ :stylers }
