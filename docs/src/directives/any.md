# `.any`

The `.any` directive interprets its inputs as conditions and iff at least one is true, it returns a true value.
It evaluates its arguments from left to right as required to resolve its value---if any are true, the rest of its arguments are not evaluated.

```emblem
.any{true}{true}{true} // returns true
.any{true}{true}{false} // returns true
.any{false}{false} // returns false
.any // Vacuously returns false
.any{true}{.error{Something went wrong}} // This returns true and never calls .error
```
