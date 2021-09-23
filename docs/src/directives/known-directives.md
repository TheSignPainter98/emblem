# `.known-directives`

This function outputs a list of known directives.
Currently, this list contains only directives for which a function has been defined, however in future this will be expanded to cover all directives which have associated styling information.
This list is also sorted in alphabetical order for convenience.

## Example -- Getting all help

The following will iterate through all known directives, outputting their [help text][help] to the command-line.

```emblem
.foreach{d}{.known-directives}:
	.echo: .help: !d
```

[help]: help.md
