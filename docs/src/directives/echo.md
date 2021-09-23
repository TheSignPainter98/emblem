# `.echo` and `.echo-on`

These directives write text to the standard output.
If multiple text arguments are given, they are concatenated together with spaces before being echoed.
The `.echo-on` directive treats its first input as a typesetting loop pass number and will only output on that pass.

## Example -- Asking the user’s name

Using the [`.readline` directive][readline] a line of input can be read from `stdin`, allowing the program to respond to the user.

```emblem
.echo: Hello, what’s your name?
!name <- .readline
.echo: Oh hello, !name
```

## Example -- Outputting the current pass

The following could be placed at the top of an input file to more explicitly show which typesetting pass is currently being evaluated.

```emblem
.echo-on{1}: Hello this is the first pass
.echo-on{2}: This is the second pass
.echo-on{3}: This is the third one
```

[readline]: readline.md
