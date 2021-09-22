# `.help`

When the user is unsure of how to use a directive, they can get some assistance by calling the `.help` directive.
This takes input of a directive-name, and returns a some associated help-text.
It should be noted that the preceding `.` from the directive name is omitted.

## Example -- Getting the help of the help function

Even the `.help` directive can be input into `.help`

```emblem
.help: help // Returns ‘Show documentation for a given directive’ (or there-abouts)
```

## Example -- Getting all help

The following will iterate through all [known directives][known-directives], outputting their help text to the command-line.

```emblem
.foreach{d}{.known-directives}:
	.echo: .help: !d
```

[known-directives]: known-directives.md
