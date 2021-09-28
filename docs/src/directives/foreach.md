# `.foreach`

The `.foreach` directive takes three inputs: an iteration variable name, a list of values and a loop body, and evaluates the loop body once for each value in the given list, assigning the iteration variable as it goes.
The list of values is parsed as a space-separated list.

As an example, the following will write the help text associated with each known directive to the console.

```emblem
.foreach{d}{.known-directives}:
	.echo: .help: !d
```

The `.foreach` directive hence performs a bounded iteration---the number of times it loops is bounded by the size of its ‘values’ input.


## Example -- Testing the truth values of a boolean function

This code will iterate over input values to the `.expr` implication operator, `=>`, and output the associated truth value.

```emblem
.foreach{x}{false true}:
	.foreach{y}{false true}:
		.echo: !x !y .expr: !x => !y
```
