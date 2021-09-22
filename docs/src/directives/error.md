# `.error` and `.error-on`

When something has gone terribly wrong, the user may wish to stop the program.
In Emblem, this can be done by use of the `.error` directive, which takes input of a message and outputs it as an error from the current source location before halting the program.
If multiple arguments are given, all are concatenated together before output is given as usual.

If the error is only applicable to a certain typesetting pass, the `.error-on` directive may be used.
This takes input of a number and message(s) as the `.error` directive, but only outputs when the current typesetting iteration is equal to the number given.

For non-critical errors, that is, those for which execution can safely continue, consider using the [`.warn` or `.warn-on` directives][warn].

## Example -- Sanitising user input

When using the [`.readline` directive][readline] which reads a single line from the standard input, some sanitisation may be needed on a user’s input.
The following will check to see that the user has indeed input something.

```emblem
.echo: Please input a number
!x <- .readline
.if{.streq{}: !x}:
	.error: User didn't enter anything
```

[readline]: readline.md
[warn]: warn.md
