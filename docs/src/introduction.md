<center>
	<img src="favicon.svg" alt="Emblem logo" width="40%">
</center>

# Introduction

When writing documentation, there are generally two choices: good looking source, and good looking output.
In spite of their ease-of-use, word-processors are typically unable to produce good-quality typographic output, with poor justification which packs only one line at a time, lax placement of figures and only rudimentary support for global document styling.
On the flip-side, high-quality typesetting programs can be unintuitive, have odd behaviours and use a notation which us unfriendly to the user.
Emblem bridges the gap between these two realities.

## What's Emblem?

Emblem is an elegant, extensible typesetter with both a friendly interface and good output.
It sports:

- A lightweight, uncluttered, unified syntax
	- With Markdown syntactic sugar
- Extension support through a simple interface
	- User-space extensions allow for the creation of arbitrary document elements
	- Driver-extensions allow for output on any device
- Excellent typographic quality output
- Global document styling through either CSS, SCSS or SASS

## Usage &amp; Document Format

Suppose we have a simple document called `hello.em`---

```markdown
# Hello, world!

How are you today? Here's a poem I like.
.centre:
	Concurrency bugs are,
	Roses are red,
	Dofficult to resolve.
	Violets are blue.

You can also write equations, which don't look to bad in the source code, such as #some-eq.
.eq: @some-eq
	y = .sqrt(2) * a_b/3^2

Hope _you_ **like** `it`.
```

This can be compiled into a PDF (the default output driver) with the following command:

```sh
em hello.em
```

Which will automatically apply the default style (`article`) and output in the default location based on the input, `hello.pdf`.
Alternatively, to typeset the above as a presentation outputted as a man page (if you wanted this I guess?) in a file `insanity.1`, this works---

```sh
em -Tman -cpres hello.em -o insanity.1
```
