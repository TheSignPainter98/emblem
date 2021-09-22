<center>
	<img src="favicon.svg" alt="Emblem logo" width="40%">
</center>

# Introduction

When writing documentation, there are generally two choices: good looking source, and good looking output.
In spite of their ease-of-use, word-processors are typically unable to produce good-quality typographic output, with poor hyphenation and justification algorithms, lax placement of figures and only rudimentary support for global document styling.
On the flip-side, high-quality typesetting programs can be unintuitive, have odd behaviours and use a notation which us unfriendly to the user.
Emblem bridges the gap between these two realities.

When many existing programs were created, some particularly useful technologies either did not exist or were in their infancy.
Emblem is designed with a more modern approach to the process of typesetting large documents, in particular, it is designed to be extensible, hackable and with support for global styling rules with CSS.

Although significant effort has been expended to make Emblem easy to use, it is expected that the user will have some familiarity with:

- Running programs from the command-line / shell
- Basic knowledge of CSS styling rules

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

```emblem
# Hello, world!

How are you today? Here's a poem I like.

.centre:
	Murder Squad, Murder Squad,
	Does whatever a murder squad does
	Can they swing from a web?
	No they can’t ... Murder Squad

Hope _you_ **like** `it`.
```

This can be compiled into a PDF (the default output driver) with the following command---

```sh
em hello
```

The [command-line interface][cli] allows for a good amount of customisation of the typesetting run.
For example, the above could be modified to apply a different base style, ‘my-article-style,’ outputting to HTML---

```sh
em hello -cmy-article-style -Thtml
```

## Remarks

This documentation has sections for all kinds of Emblem users, from document- and extension-writers to Emblem-developers.
No user is expected to require all of the information in these docs, hence they are split into separate sections which contain:

- [Information on how Emblem works][how-it-works]
- [Information on how to use Emblem][how-to-use-emblem]
- [Descriptions of the directives available by default][directives]
- [Information on the CLI, extension and core APIs][api]
- [Licensing information][license-info]

[api]: api.md
[cli]: generated/command-line-args.md
[directives]: directives.md
[how-it-works]: how-emblem-works.md
[how-to-use-emblem]: how-to-use-emblem.md
[license-info]: license-and-author-notes.md
