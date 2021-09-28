# `.expr`

The `.expr` directive takes input of an [expression][expression], parses and evaluates it.
As it uses the same parsing logic as the [flow-control][flow-control] directives, it is possible to use `.expr` for debugging.

## Example -- A simple calculator

Using `.expr` and a while loop, a simple calculator can be made.
The following will repeatedly read an expression to evaluate from the user, until they input nothing.

```emblem
!resp <- .readline
.while{! .streq{}{!resp}}:
	.echo: .expr: !resp
	!resp <-- .readline
```

[expression]: ../expressions.md
[flow-control]: ./expl/flow-control.md
