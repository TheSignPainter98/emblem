# `.readline`

The `.readline` directive reads a single line from standard input and returns it.

## Example -- Interacting with a user

As `stdin` is read, the `.readline` directive can be used to interact with a user.

```emblem
.echo: Ahoy there! What’s yer name?
!resp <- .readline
.echo: Ahoy there, !resp
```
