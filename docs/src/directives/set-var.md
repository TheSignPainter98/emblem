# `.set-var`

The `.set-var` directive is what the `!var <- val` syntactic sugar is translated to at parse-time.
This directive takes the name of a variable and a value, and assigns that variable to the _string representation_ of the given value at the point of call.

By syntactic sugar, the following two lines are equivalent and both set the value of variable `hello` in the current scope to ‘world.’

```emblem
.set-var{hello}{world}
!hello <- world
```

Note the initial exclamation mark required on the second line.
This is required by the parser and is reflected in the same number of exclamation marks when referencing that variable.

If the variable name contains exclamation marks, the scope of the assignment is widened, so for example, to set the variable `hello` in the _parent_ scope and set that to ‘world,’ we can call

```emblem
.set-var{\!hello}{world}
```

Note that as we want the literal variable name, the initial exclamation mark must be escaped to prevent variable recognition at parse-time and hence expansion.
Only the first exclamation mark requires an escape, so the literal `!!asdf` can be escaped into `\!!asdf`.
The above snippet is equivalent by syntactic sugar to the following.

```emblem
!!hello <- world
```

More exclamation marks will widen the scope-search further.

The `.set-var` directive has some variants for some specialised use-cases.
We combine [`.expr`][expr] with `.set-var` with the `.set-var-expr` directive, which evaluates the value parameter as an expression before performing assignment.
We also have `.find-set-var` and `.find-set-var-expr`, which are the same as `.set-var` and `.set-var-expr`, except that the search for an existing instance of the variable is always performed, even when the number of exclamation marks specified would not trigger it with the non-find assignment directives.
This cleans up the syntax slightly for the common operation of extracting a value from the scope of a loop for example---the following two are equivalent.

```emblem
!!hello <- world
!hello <-- world
```

The variants of `.set-var` each have their own syntactic sugar.
The following is a list of all sugars associated with assignment

```emblem
!var <- value // Equivalent to .set-var{var}{value}
!var <~ exp // Equivalent to .set-var-expr{var}{exp} or !var <- .expr: exp
!var <-- value // Equivalent to .find-set-var{var}{value}
!var <~~ exp // Equivalent to .find-set-var-expr{var}{exp} or !var <-- .expr: exp
```

## Example -- Recording a user’s name for later use

We could write a document like so which takes input of the user’s name and then re-uses that value multiple times.
By storing the user’s response in a variable, we can avoid the need to re-ask them.

```emblem
.echo: Please enter your name
!name <- .readline
Dear !name,
It has come to my attention that the name !name is shared by us both.
Please change yours.
Warmest regards,
!name
```

[expr]: expr.md
