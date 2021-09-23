# Moonscript/Lua Extension API

Emblem is hackable, that is, arbitrary functionality may be added by its users.
This is done in one of three ways:

1. In code executed as the document goes through its typesetting cycle (‘extensions’)
2. In code executed to convert a given input format to Emblem’s internal structures (‘input drivers’)
3. In code executed to convert to a given output format (‘output drivers’)

In this section, we will explore the Emblem standard library, `std.*`:

- [`std.ast`](generated/ext/lib/std/ast.moon.md)
- [`std.base`](generated/ext/lib/std/base.moon.md)
- [`std.bib`](generated/ext/lib/std/bib.moon.md)
- [`std.constants`](generated/ext/lib/std/constants.moon.md)
- [`std.events`](generated/ext/lib/std/events.moon.md)
- [`std.func`](generated/ext/lib/std/func.moon.md)
- [`std.hdr`](generated/ext/lib/std/hdr.moon.md)
- [`std.in.*`](modules/moon/std.in.md)
- [`std.lingo`](generated/ext/lib/std/lingo.moon.md)
- [`std.log`](generated/ext/lib/std/log.moon.md)
- [`std.out.*`](modules/moon/std.out.md)
- [`std.ref`](generated/ext/lib/std/ref.moon.md)
- [`std.show`](generated/ext/lib/std/show.moon.md)
- [`std.store`](generated/ext/lib/std/store.moon.md)
- [`std.style`](generated/ext/lib/std/style.moon.md)
- [`std.util`](generated/ext/lib/std/util.moon.md)
<!-- - [`std.edit`](generated/ext/lib/std/edit.moon.md) -->

---

## Typesetting-time extensions

Typesetting-time extensions, hereafter referred to simply as ‘extensions,’ are snippets of [Lua][lua] code which are imported after the document has been parsed and are executed as it undergoes its typesetting run.

### Emblem Public Table

Extensions can define functions to be accessed in the document by editing the Emblem Public Table stored in the `em` variable.
For example, we may wish create a document which includes a note of how long it took to compile.
To do this, we must create a function which creates the desired string, and place it into some field in the `em` table, say `cpu_time_used`

```lua
em.cpu_time_used = function()
	return 'This document was typeset in ' .. os.clock() .. ' seconds of CPU time'
end
```

If this code is written in a file called `cpu-time.lua`, it can be imported by adding the flag `-fcpu-time` when `em` is run (note the file extension is optional).
Now, when the directive `.cpu_time_used` is seen in the document, the above code will be executed, it will be replaced with the message.

### Evaluation Order

Emblem is lazy, that is, it tries to do no more work than is necessary to typeset a document.
So for example, if a directive takes three inputs and just outputs the first one, emblem will not bother to evaluate the others.
This is because by default, Emblem will only evaluate a node if it can guarantee that it will appear in the output.

Sometimes, however, it can be useful to force Emblem to use a different evaluation order, such as to inspect the results which would not otherwise appear directly in the output.
This can be done using the `eval` function, which takes a node and evaluates it, or the `eval_string` function which
Evaluation order is manipulated in the definition of the `.if` directive, which looks something like the following:

```lua
local base = require('std.base')
base.em['if'] = function(c, b)
	cs = base.eval_string(c)
	if cs == '' or cs == '0' or string.lower(cs) == 'false' then
		return nil
	else
		return b
	end
end
```

Here, although input `c` is never present in what is returned, by calling `eval_string` upon it we can reason about it.

### Events

To help extensions react to how the document is being processed, there are several events which are triggered.
These are triggered on objects which extend the `Component` class defined in the `std.base` module and are as follows.

1. `on_start`, executed once after all extensions are loaded
2. `on_iter_start`, executed at the start of each iteration
3. `on_iter_end`, executed at the end of each iteration
4. `on_end`, executed once, after the final iteration but before output

There are a number of classes which may be imported from the `std.std` module which provide frameworks for storing data whilst reacting to these events.
For example, the table of contents is a subclass of `Component` which stores the names and numbers of headings as the document is processed, requesting another run if the table of contents at the end of the previous run is different to that at the end of the current (e.g. a page-number has been updated by some other change).

A re-iteration can be requested by calling the `requires_reiter` function in [Lua][lua].
This will cause the typesetting loop to be run again, unless the (user-configurable) number of maximum iterations has been reached.
The number of the current iteration (starting from 1) is accessible through the `em_iter` variable.

## Input Drivers

Emblem can take input of any format for which it has an input driver.
When Emblem inputs a file through the `.include` directive, a language can optionally be specified in the second parameter to determine the parser to use.
This language is used to look up the parser to use in the `std.in.drivers.known_languages` table.
An input driver is simply a module which adds at least one parser function into this table.

## Output Drivers

Emblem is capable of outputting to any format for which it has an output driver.
The binary itself contains some output drivers, but it is also possible to import ones from other sources as desired.
In analogy with [input drivers](#input-drivers), there exists a table, `std.out.drivers.output_drivers`, which is used when looking up the language into which the document will be output.
An output driver is simply module which adds at least one outputting function into this table.

[lua]: https://www.lua.org
