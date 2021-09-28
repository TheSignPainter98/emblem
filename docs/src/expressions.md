# Expressions and Conditions

In Emblem, an _expression_ is a section of text which is parsed and evaluated as a mathematical statement.
These can be used to concisely represent maths operations without using the [arithmetic][arithmetic-directives] or [logic][logic-directives] directives.
A _condition_ is a special case of an expression, where if the expression is empty, false is returned instead of emitting a parse-error.
These are commonly used in the [flow-control][flow-control] directives.

It must be noted that expressions are _only capable of handling numbers,_ so for checking textual equality for example, the [`.streq` directive][streq] must still be used.

The following operators are recognised in expressions with the numbers giving their precedence level with higher numbers indicating that the associated operators bind more tightly.
As the multiplication operator, `*`, has a higher precedence than the binary subtraction operator, `-`, we can see that the expression `1+2*3` is correctly evaluated to 7 instead of 9.

1.
	- `<==>` -- Dual implication
	- `==>` -- Single-implication (right)
	- `<==` -- Single-implication (left)
1.
	- `||` -- Disjunction (‘lazy-or’)
1.
	- `&&` -- Conjunction (‘lazy-and’)
1.
	- `<=>` -- Dual implication (higher precedence)
	- `=>` -- Single implication (right, higher precedence)
1.
	- `!=` -- Numeric inequality
	- `==` -- Numeric equality
1.
	- `<=` -- Inequality, less-than-or-equal, lazy
	- `<` -- Inequality, less-than, lazy
	- `>=` -- Inequality, greater-than-or-equal, lazy
	- `>` -- Inequality, less-than, lazy
1.
	- `~` -- Bitwise exclusive disjunction (‘xor’)
	- `|` -- Bitwise disjunction (‘strict-or’)
1.
	- `&` -- Bitwise conjunction (‘strict-and’)
1.
	- `+` -- Addition
	- `-` -- Subtraction
1.
	- `//` -- Integer division
	- `%` -- Modulo
	- `*` -- Multiplication
	- `/` -- Division
1.
	- `!` -- Logical negation
	- `-` -- Numeric negation
	- `~` -- Bitwise negation
	- `+` -- Unary plus
1.
	- `^` -- Exponentiation

## Example -- Fizz-Buzz

The following is a solution in Emblem to the [FizzBuzz][fizz-buzz] common interview problem.
This is by no means a model solution for Emblem is not a programming language---first and foremost it is a typesetter, it just happens to have a handy extension language which can be programmed in.

```
.for{!i <- 1}{1 <= !i <= 100}{!i <~ !i + 1}:
	.if{!i % 15 == 0}:
		.echo: FizzBuzz
	::
		.if{!i % 5 == 0}:
			.echo: Buzz
		::
			.if{!i % 3 == 0}:
				.echo: Fizz
			::
				.echo: !i
```

[arithmetic-directives]: directives/expl/arithmetic.md
[flow-control]: directives/expl/flow-control.md
[logic-directives]: directives/expl/logic.md
[streq]: directives/equality.md
[fizz-buzz]: https://www.wikiwand.com/en/Fizz_buzz
