# `.call`

The `.call` directive takes input of a directive name and some arguments to pass to it.
It returns a call to the specified directive with the arguments given.
For example, one could call `.echo` with three arguments like so---

```emblem
.call{echo}{Hello}{world}{how are you?}
```

Which is implicitly translated by the `.call` to---

```emblem
.echo{Hello}{world}{how are you?}
```

As the _name_ of the directive is required, note that the `.` which precedes directive calls is omitted, so `.echo` above becomes just `echo`.

## Example - Specifying an error function

The following code could be used to change the severity of an error condition based on a user’s preference.
That the user inputs a number is assumed for brevity.

```emblem
!err_severity <- .readline
!err_func <- .case{!err_severity}{echo}{warn}{error}
...
.call{!err_func}: Something went wrong!
```
