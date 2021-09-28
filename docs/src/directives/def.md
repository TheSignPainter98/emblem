# `.def`

This directive allows the user to define their own directives for later use.
It takes two inputs: a directive name and a directive body to be executed whenever the new directive is called.
The new directive is allowed to take parameters which can be referenced by values `!n` in the loop body, so `!1` evaluates to the first argument given, `!2` the second and so on.

Note that as a directive name is taken, there is no `.` preceding the new directive name when `.def` is called.
If functions are nested, it may be useful to use [multiple exclamation marks][get-var] when accessing arguments from different scopes.

## Example -- Checking input

The following could be used to perform type-checking upon a given value.
```emblem
.def{bool}:
	.if{! .streq{!1}{true} || .streq{!1}{false}}:
		.error: Expected either ‘true’ or ‘false’ but got !1
	!1
```

Then, later in the document the following could be written:

```emblem
.bool{true}
.bool{false}
```

This would give certainty that boolean values have indeed been given.
The `.bool` function may be useful if some values are stored in variables, or some other method which obscures the value from the source.

The `.def` directive is provided to allow an author to extend the functionality of Emblem from within their document.
It is useful for simple tasks, however for more complicated ones, it may be easier to write an extension and make use of [Moonscript][moonscript] or [Lua][lua]’s abstractions instead.

[get-var]: get-var.md
[lua]: https://www.lua.org
[moonscript]: https://moonscript.org
