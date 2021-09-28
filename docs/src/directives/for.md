# `.for`

The `.for` directive emulates the standard [for-loop][for-loop] seen in many programming languages.
It takes input of:

1. An initialiser -- a statement which is performed before iterations commence. It is customary to define an iteration variable here.
2. A condition -- a [condition][condition] which is evaluated before each iteration, and if it is found to be false, `.for` execution terminates
3. A mutator -- a statement which is executed at the end of each iteration. It is customary to mutate the iteration variable here.
4. A loop body -- a section of emblem script to execute each iteration which is included in the for-loop’s return value

There are two scope considerations:

- The loop body is evaluated within its own scope
- The initialiser, condition and mutator are evaluated within a scope which surrounds the loop body.

This second point means that whereas the default assignment operator, `<-`, can be used in the initialiser and mutator, for the same effect on the iteration variable, the finding-assignment operator, `<--`, should be used to ensure that the change to the iteration variable is not lost at inner scope-closure.
The same is true of the expression-assignment and finding-expression-assignment operators, `<~` and `<~~`.

## Example -- Iterating over even numbers

The following outputs the even numbers inclusively between one and one hundred by allowing an iteration variable, `!i` to take values from 2 to 100 in increments of 2.

```emblem
.for{!i <- 2}{1 <= !i <= 100}{!i <~ !i + 2}:
	.echo: !i
```

[condition]: ../expressions.md
[for-loop]: https://www.wikiwand.com/en/For_loop
