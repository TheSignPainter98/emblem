# `.div`

Takes two numbers, returns the left divided by the right.
There are some special cases:

- When only the right input is zero:
	- Returns \\( +\infty \\) when the left input is positive
	- Returns \\( -\infty \\) when the left input is negative
- When both inputs are zero:
	- Returns `±NaN`

The `.div`, `.idiv` and `.mod` functions are related by the following identity---

\\[ \frac ab = a \mathbin{//} b + \frac{a \mathbin{\\\%}b} b \\]
