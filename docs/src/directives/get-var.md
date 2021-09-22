# `.get-var`

This directive takes input of a variable name and obtains its value in the current scope.
If there is no such value with that name, nothing is returned.

The `.get-var` directive has associated syntactic sugar, so the following two are equivalent.

```emblem
.get-var{variable}
!variable
```

More exclamation points can be used to widen the scope searched, hence `!!var` will obtain the value of ‘var’ in the scope above its current definition.

The `.get-var` directive is the counter-part to [`.set-var`][set-var], which sets variables to given values.

[set-var]: set-var.md
