# `.exists`

The `.exists` directive checks whether a given variable name exists in current scope or in a parent and then returns `1` or `0` accordingly.

```emblem
.exists{x} // Returns 0 as !x does not exist
!x <- asdf
.exists{x} // Returns 1 as !x now exists.
```
