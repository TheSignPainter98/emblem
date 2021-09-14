# `std.out/`

This module collection provides the machinery for defining and implementing the core set of output drivers.

Its most important file is [`std.out.drivers`](../../generated/ext/lib/std/out/drivers.moon.md), which contains the code to translation to output formats and writing to files, and defines some useful classes for output drivers.
All other modules in this module collection are implementations of input drivers which are always available when running Emblem.
