# `.while`

The `.while` directive takes two inputs: a condition and a loop body and does the following:

1. Check to see if the condition is true
2. If it is, evaluate the loop body and go to step 1

The results of each iteration are concatenated together beneath a Content node:

## Example -- Sanitising user input

When using the [`.readline` directive][readline] which reads a single line from the standard input, some sanitisation may be needed on a user’s input.
The following will ask the user for an input until they enter something which isn’t empty
```emblem
.echo: Please input a number
!x <- .readline
.while{.streq{}: !x}:
	.warn: User didn't enter anything
	!!x <- .readline
```

[readline]: readline.md
