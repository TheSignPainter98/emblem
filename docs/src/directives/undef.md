# `.undef`

Calling `.undef` with the name of a directive removes that directive from use.
If the given directive is not known, nothing is done.

## Example -- Preventing system calls

The following example un-defines the [system directive, `.$`][system-directive].

```emblem
.undef{$}
```

## Example -- Ruining user experience

The `.undef` directive can be used to remove all directives from availability.

```emblem
.foreach{d}{.known-directives}:
	.undef{!d}
```

[system-directive]: system.md
