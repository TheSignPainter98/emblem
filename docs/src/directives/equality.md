# `.eq`, `.numeq`, `.streq`

In Emblem, there are three modes of equality: structural, textual and numerical.
These correspond to the equality functions `.eq`, `.streq` and `.numeq` respectively.
Although all compute equality, the differences between each should be noted.

## Structural equality -- `.eq`

This definition of equality is the strongest, it looks at the document structure of two items and asks whether they are _identical,_ that is, not only does it look at each literal fragment, but also checks whether these are arranged in memory in the same way.

It must be noted that _results_ of directive calls are also checked, which can lead to some odd results.

### Example -- Identical structures

The following examples all return true as they are identical

```emblem
.eq{}{}
.eq{hello, world!}{hello, world!}
.eq:
	.it:
		Hello, world!
::
	.it:
		Hello, world!
```

### Example -- Non-equal, apparently identical structures

However, all of the following are false as, although they may appear the same in output they differ in how they got there.
Some of the following behaviour may seem strange, but comes as a consequence of the very insistent nature of this kind of equality.

```emblem
!x <- a
!y <- b
.eq{!x}{!y} // This is false as different variables are referenced (‘x’ and ‘y’)
.eq: // This is false as when evaluated these two sections are given distinct numbers
	# Hello, world!
::
	# Hello, world!
.eq: // This is false due to the method by which trailing arguments are handled (a node to represent hold possibly multiple lines is necessarily inserted
	.it{Hello, world!}
	.it:
		Hello, world!
```

### Example -- Apparently non-equal, identical structures

As syntactic sugar is translated into directive calls when parsing is performed, all of the following are true.

```emblem
.eq{_something emphasised_}{.it{something emphasised}}
.eq{[a-citation]}{.cite{a-citation}}
.eq{!x <- asdf}{.set-var{x}{asdf}}
```

## Textual equality -- `.streq`

Whereas structural equality checks the internal representation of structures, textual or ‘string’ equality simply checks to see whether the literal text from two sources is the same, regardless of how they were constructed.

Any structurally-equal pair of values is necessarily textually-equal.

### Example -- Non-identical structures with textual equality 1

The following all resolve to a true value at the point when the standard library has finished loading.

```emblem
.streq{**something emphasised**}{_something emphasised_}
.streq{.tt{something}}{.bf{something}}
.streq{.it{two words}}{.it{two}{words}}
```

### Example -- Apparently-equal structures without textual equality

The following evaluates to a false value for the same reason that it does not have structural-equality---when constructing a `.h1` header, a section number is prepended
```emblem
.streq:
	// The text here is ‘1. Hello, world!’
	# Hello, world!
::
	// The text here is ‘2. Hello, world!’
	# Hello, world!
```

## Numerical equality -- `.numeq`

Whereas textual equality checks that all literals in the text are equal, numerical equality first attempts to translate the text into a number before doing the same.
As a result, numerical equality is weaker than textual equality.
The numbers considered are base-10.

It should be noted that the numerical equality function does not perform any operations such as addition or subtraction.
If these are required, they must be invoked explicitly through other directives whose results are passed as inputs to `.numeq`.
[More information on arithmetic directives is available elsewhere.][arithmetics]

### Example -- Equal numbers

Prepended zeros have no bearing on the value of a number and hence are ignored under numerical equality.
All of the following evaluate to a true value.

```emblem
.numeq{1234}{1234}
.numeq{1234}{000000000000000001234}
```

[arithmetics]: ./expl/arithmetic.md
