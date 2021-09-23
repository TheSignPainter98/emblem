# `.all`

The `.all` directive interprets its inputs as conditions and iff none is false, it returns a true value.
It evaluates its arguments from left to right as required to resolve its value---if any are false, the rest of its arguments are not evaluated.

```emblem
.all{true}{true}{true} // returns true
.all{true}{true}{false} // returns false
.all{false}{false} // returns false
.all // Vacuously returns true
.all{false}{.error{Something went wrong}} // This returns false and never calls .error
```
