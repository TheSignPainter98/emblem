# .call

## Example - Warn or error

```emblem
!something <- .$: fhjdkslfha
!err_func <- echo
.if{!something}:
	.call{!err_func}: Something went wrong!
```
