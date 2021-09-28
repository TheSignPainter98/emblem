# `.defined`

The `.defined` directive checks whether a given directive name corresponds to a known directive function and returns `1` or `0` accordingly.

```emblem
.defined{echo} // Returns 1 as .echo exists
.undef{echo} // Undefines .echo
.defined{echo} // Returns 0 as .echo no longer exists.
```
