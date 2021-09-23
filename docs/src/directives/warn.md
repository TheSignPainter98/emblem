# `.warn` and `.warn-on`

When something has gone wrong but the problem isn’t fatal, a warning may be required to notify the user of what has happened.
This can be done using the `.warn` directive, which takes input of a message and outputs it as a warning from the current source location.
If multiple arguments are given, all are concatenated together before output is given as usual.

The `.warn` directive is subject to the warning-handling specified at the command-line, in particular the [`-E` flag][warn-to-error-flag] turns warnings into errors.

If the warning is only applicable to a particular typesetting pass, the `.warn-on` directive may be used, which takes input of a pass number to perform the output upon and the remainder of its arguments as `.warn`.

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
[warn-to-error-flag]: ../generated/command-line-args.md#-e---fatal-warnings
