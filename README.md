# Emblem

An elegant, extensible typesetter with a friendly interface and decent output.

## Table of Contents

<!-- vim-markdown-toc GFM -->

* [Overview](#overview)
* [Usage](#usage)
* [How to Obtain](#how-to-obtain)
	* [Using Existing Binaries (simple)](#using-existing-binaries-simple)
	* [Building Binaries from Scratch (more complicated)](#building-binaries-from-scratch-more-complicated)
* [How it Works](#how-it-works)
* [Syntax Overview](#syntax-overview)
	* [Basic Syntax](#basic-syntax)
	* [Syntactic Sugar](#syntactic-sugar)
	* [Emblem Script](#emblem-script)
		* [Variables](#variables)
		* [Functions](#functions)
		* [Control flow](#control-flow)
	* [Preprocessor Directives (‘Pragmas’)](#preprocessor-directives-pragmas)
* [Styling Overview](#styling-overview)
* [Extensions Overview](#extensions-overview)
	* [Typesetting-time extensions](#typesetting-time-extensions)
		* [Emblem Public Table](#emblem-public-table)
		* [Evaluation Order](#evaluation-order)
		* [Useful Functions](#useful-functions)
		* [Useful Values](#useful-values)
		* [Events](#events)
		* [Useful Classes](#useful-classes)
	* [Output Drivers](#output-drivers)
* [License and Author](#license-and-author)
* [Contributions](#contributions)

<!-- vim-markdown-toc -->

## Overview

Emblem takes input of a file written in the format it recognises, typesets it and then outputs it in some format.
Whilst typesetting, the document is styled using [CSS][css-ref] and can be operated upon by extensions written in [Lua][lua].
The resulting document is then passed to some output driver which is responsible for translating it into some format and outputting it.

As there can be relations between some parts of a document, the running of [Lua][lua] extensions, styling by [CSS][css-ref] and typesetting is run repeatedly until either the document settles to an acceptable state, or a maximum number of iterations is reached.
As this iteration is done internally, the executable `em` need only be run once to fully typeset a document from a given input.
Emblem operates as below.

![How Emblem operates](https://raw.githubusercontent.com/TheSignPainter98/emblem/master/docs/how-it-works.svg)

## Usage

Emblem can be used through command-line using the `em` binary.
For example to compile a document `hello.em` into HTML, the following can be run.

```bash
em example
```

Note the ‘.em’ file extension is optional, this will be true of most file extensions used.

The output format can be changed by passing the `-T` option.
By default, `html` is used, but other formats are also available.
For example, the output could be changed to bbcode:

```bash
em example -Tbb
```

Emblem also supports extensions which can be added to the document-environment by the `-f[ext]` option.
Extensions can also be passed text values using the `-a[ext].[param]=[val]` option.
For example, `example.em` could be compiled into bbcode using the `asdf` extension with its argument `fdsa` being set to `qwer`:

```bash
em example -Tbb -fasdf -aasdf.fdsa=qwer
```

The order of options passed is not significant, except in the case that the same parameter to the same extension is set multiple times, in which case the rightmost value is taken.
Documentation for the entire command-line interface are available in the [releases][releases-page].

## How to Obtain

To use Emblem, you will need the `em` binary, which can be obtained through one of two methods.

### Using Existing Binaries (simple)

Binaries for Linux, ~~macOS~~ and ~~Windows~~ can be found on the [releases page][releases-page]. _Coming soonish_

~~For Arch Linux users, Emblem is available via the AUR:~~ _Also coming soonish_

```bash
yay -S emblem
```

### Building Binaries from Scratch (more complicated)

Programs you will need:

- [`git`][git]
- [`yq`][yq]
- [GNU Autotools][gnu-autotools]
- [Standard UNIX tools][unix-tools]
- [gcc][gcc]

To compile and run the `em`, you will need to install the following C-libraries:

- [criterion][criterion]
- [glibc](https://www.gnu.org/software/libc/) (Comes with all good GNU/Linux distributions)
- [libcss](https://www.netsurf-browser.org/projects/libcss/)
- [libsass](https://sass-lang.com/libsass)
- [lua](https://www.lua.org/download.html) 5.4
- [lua-argparse](https://luarocks.org/modules/argparse/argparse)
- [lua-lyaml](https://luarocks.org/modules/gvvaughan/lyaml)
- [moonscript](https://moonscript.org)
- [ncurses](https://invisible-island.net/ncurses/)

Now to compile, clone the git repository, `cd` into it, and then issue the following commands.

```bash
./scripts/autogen
./configure
make
```

This should result in a binary called `em` being created.

Maintainers: When adding new source files, the above commands will need to be re-run as the `autogen` script generates a list of files which will be linked into the binary.
To execute tests, run `make check_em` and then `./check_em`.

## How it Works

Here we discuss how Emblem works and how it may be used.

To start, Emblem takes input of a single file—either given explicitly at the command-line or otherwise implicitly taken to be read from its standard input stream.
Emblem expects this file to conform to the rules of its syntax as specified [a later section](#syntax-overview).
From this file may be referenced others which Emblem will then read and parse as it did the first.
This process completes once there are no required files which have not been read and parsed.

Extensions are then loaded and initialised.

Next, the typesetting loop starts.
This begins with a run through the document evaluating calls to functions defined in extensions, these extensions may indicate that they require another typesetting run.
The result is then analysed to ascertain the styles which must be applied to each element in the document.
The document is then typeset.
If another typesetting run has been requested and the number of iterations has not exceeded the maximum specified by the user, the loop reiterates.

Extensions are then de-initialised.

Finally, the typeset document is passed to the output driver which is responsible for translating the structure into an external document format.


## Syntax Overview

The Emblem syntax is based on two core concepts: directives and pragmas, the former being a more expressive cousin of the latter.
These are both commands issued by the user which can affect the style and content of a document but are processed at different stages in the processing of a document.

Pragmas, or preprocessor directives, are commands which are executed as the document is being parsed.
They are used to change the state of the parser or force it to insert the contents of a new file at a particular point in a containing file.

Directives are used to both style content and to generate it by interacting with extensions.
All other concepts in the Emblem syntax such as markdown-like syntax or the scripting language are translated down to directive-calls for evaluation.

Each of these things shall be explained in further detail over the coming subsections.

### Basic Syntax

```emblem
As one might expect, in Emblem, words are words.
```

If a line contains a double forward-slash, the rest of the line is interpreted as a comment and will not appear in the output.
Multi-line comments are delimited by `/*` and `*/`.

```emblem
// This is a comment.
/* This is a multi-line comment.
   Like the name implies, it can exist over multiple lines. */
/* These comments can also be nested /* like so, without risk of the comment being */ prematurely closed when it is interpreted. */
```

Elements of a document can be styled by using directives.
These are words which start with a dot, such as `.example` and are the link to the more advanced features of Emblem.

A directive is both a styling instruction and also—if it has been declared by an extension—a call to a function which takes some parameters and outputs document content.
These arguments can be specified in a few different ways.
All of the following will centre a piece of text.

```emblem
.centre{This text will be centred}
.centre{As 'centre' is not a command,}{the different arguments will just be concatenated in the output}
.centre{It’s possible to use the curly braces to hold multi-line arguments.}{
	Just so long as the contents in the curly braces is indented.
	This will continue until the matching closing-brace.

	Paragraph breaks are allowed and are applied if the context permits.
}
.centre:
	A single colon can be used to start an indented block as an argument.
	This is exactly the same as the curly-braces and indents above.

	Just with a little-less visual clutter.
.centre{Curly brace arguments}:
	and those with a colon and indent can be combined, the only constraint is that the curly-brace arguments (if any) must come first.
.centre:
	If there are multiple long arguments to a function, it can be useful to use a _continuation_ argument.
	These are delimited by a dedented double colon,
::
	like so. These continuation arguments follow the same rules as those above except that they can only appear after a single-colon argument block.
```

The ability to delimit multiple sections of a document can be useful when the directive is a recognised function call.

```emblem
.if{true}:
	Something to show when the condition is true
::
	Something to show when the condition is false
```

Conditions are interpreted by the following rules: if the condition is empty or equal to either `0` or `false` (case-insensitive), then it is interpreted as ‘false.’ otherwise it is ‘true.’
This is an example of the scripting module, `std.lingo`.
More information is seen in the [scripting reference](#emblem-script).

Emblem interprets some characters as special (for example the underscore which can be used to make italics), however in some cases their literal value is desired instead.
In this case, a backslash can be used to force Emblem to ignore special meanings (e.g. `\_` will always output an underscore, instead of possibly affecting styling).

### Syntactic Sugar

The [basic syntax](#basic-syntax) can be used to express all concepts which Emblem can understand, however for convenience, some more brief forms are defined.
Some of the following should be familiar to markdown users.

```markdown
_Single underscores denote italic._
*So do single asterisks.*
__Double underscores denote bold.__
**So do double asterisks.**
_**Styles can be nested, nested**_
__And *nested* some more__
`backticks can be used use monospace`
`These too _can_ be **nested**`
=Single equals-delimiters make the included text _small caps_=
==Double equals-delimiters use an _alternative_ font face, such as sans-serif in an otherise serif document==
```

The above syntax simply expands to directive calls:

- Italic delimiters expand to `.it`
- Bold delimiters expand to `.bf`
- Mono-space delimiters expand to `.tt`
- Small-cap delimiters expand to `.sc`
- Alternative face delimiters expand to `.af`

It should be noted that these delimiters are only recognised at the beginning and ends of words and as such do not need to be escaped when they appear strictly within them.
The only exception to this is when trailing punctuation is used, in which case closing delimiters can be placed before these.
This can be used for example to end a non-bold sentence with a bold word but not a bold full-stop.

Headers can be specified in a line starting with one to six hashes.

```markdown
# This is a level 1 header
## This is a level 2 header, a little smaller probably
### This is a level 3 header
#### Guess what this is
##### This is a level 5 header
###### This is a level 6 header
```

These respectively expand to calls to `.h1` to `.h6`.

In Emblem, these represent numbered headings which appear in a table of contents.
If this is not desired, there are starred versions of the functions available.

```emblem
#* This is a level 1 unnumbered heading
##* This is a level 2 unnumbered heading
###* This is a level 3 unnumbered heading
####* Now guess what this is
#####* This is a level 5 unnumbered heading
######* This is a level 6 unnumbered heading
```

These expand to calls to `.h1*` to `.h6*` respectively.

### Emblem Script

Sometimes to can be convenient to affect the structure of the document in a small way without needing to write extension code.
This can be achieved by using the scripting directives define in Emblem’s standard library, which create a small bash-like language.
The scripting language contains a minimal set of directives to perform common programming language functionality.

#### Variables

The `.set-var` function sets a variable to a given value.
For example, `.set-var{x}{asdf}` will set variable `x` to the string `asdf`.

The value of a variable can be retrieved with the `.get-var` function.
After the above, `.get-var{x}` will now return `asdf`.

If it is no longer required, a variable can be undefined by calling `.undef-var`.
Any subsequent uses of the variable will return the empty string until it is redefined.

#### Functions

Emblem is quite accepting of the number of arguments passed to a function, following the standard behaviour of [Lua][lua].
If an emblem directive expects a _n_ arguments and receives too many, the rest are ignored.
Similarly, if fewer than _n_ are received, then the remaining ones are treated as [Lua][lua]’s `nil`, which if seen by Emblem’s core is expanded to the empty string.

Below, where ‘string representation’ is seen, this should be interpreted as Emblem evaluating the value it is given before concatenating together all text-elements contained within it.

The following functions are provided as standard.

- `.streq`
  This takes two inputs, checks their string representations and returns `1` if they are the same, otherwise `0`
- `.echo`
  This takes any number of inputs and prints them on the command-line, separated by spaces.
  Useful for debugging.
- `.echo-on-pass`
  This takes a number and then any number of inputs and if the current pass index is equal to the given number, executes `.echo`.
- `.defined`
  This takes one input and if its string representation is the name of a _variable_ which has previously been set returns `1`, otherwise `0`
- `.exists`
  This takes one input and if its string representation is the name of a known _function_ returns `1`, otherwise `0`

#### Control flow

The content of a document can be conditionally-modified using Emblem script directives.

The `.if` directive takes two inputs.
The first input is evaluated as a condition and if it is true, then the second input is returned, otherwise `nil`.

The `.ifelse` directive is similar to `.if` except that it takes _three_ inputs.
If the condition in the first input is evaluated to true the second input is returned, otherwise the third.

The `.while` directive represents an unbounded loop.
It takes two inputs: a condition and a body.
The `.while` loop evaluates the condition, and if it is true it evaluates the body.
This is repeated until the condition becomes false.
What is returned is the sequence of values that where returned when evaluating the body during iteration.

The `.foreach` directive represents a bounded loop.
It takes three inputs: the name of a variable, a sequence of values (such as a sentence) and a body.
The loop iterates once for each value in the sequence, assigning the named variable that value before evaluating the body.
Like the `.while` loop, what is returned is the sequence of values which were returned when evaluating the body during iteration.
If the iteration variable was already defined before the start of the loop, it retakes its previous value once the loop has ended.

### Preprocessor Directives (‘Pragmas’)

Some document-functions of Emblem do not interact with extensions.
These are preprocessed-out before the document begins its typesetting and extension-execution run.
The following statements are only valid _as the first and only element of a line._
Any white-space to the left of a pragma is ignored, hence they do not need to follow the indentation rules of the rest of the document.

The `:include` pragma takes input of a file name surrounded by double-quotes, reads it and substitutes it into the document in the current position, hence allowing for multi-file documents.
As this is handled before any execution of extension code, it is possible for external tools to interpret the file-structure of a given document without needing to execute its code.
The line:

```
:include "some-file-to-include"
```

Will include a file “some-file-to-include.em”.
The `.em` file extension is optional.

The `:line` pragma updates the parser’s reference for its current position in the document for when errors are reported.
This is useful when an Emblem document is the output of another program and the user wishes for error messages to refer to another source location.
The pragma takes a filename in double-quotes, a number which represents the new current line index and a number which represents the new current column index.
The line:

```
:line "some-other-file.md" 32 54
```

Will make Emblem believe that it is currently in “some-other-file.md” at line 32 and column 54 once the next line starts.
File extensions are not optional here as the filename is copied verbatim.
Line pragmas are local the file which is currently being parsed and so are unaffected by `:include`-ing files.

## Styling Overview

Directives can not only control the content of a document, they can style it too.
Whenever a directive is used, it automatically applies the style of the same name to its output.
For example, the `.h1` directive for defining titles also applies the `.h1` style to its output.
But where do these styles come from?

Emblem takes input of a stylesheet written in either plain [CSS][css-ref], [SASS or SCSS][sass-scss-ref].
So for example, the following snippet would make the entire document use the [Bodoni\* 11][bodoni-star] font and use light grey text on a dark-grey background.

```scss
.body {
	font-family: 'Bodoni* 11', serif;
	background-color: #2d2d2d;
	color: #dddddd;
}
```

When writing styles, care should be taken to ensure that the names of rules always start with a ‘.’, just like the directives which call them.
So for example, although using a styling rule for `body` will function as intended for HTML output, however if another format were used, the styling associated with `body` (no prefixed dot) would be missing.
To style the entire document, a `.body` style should be used instead.

Extensions can also help style their own results by importing their own stylesheets.
As these are imported before the user’s one, default styles can be overridden as desired.

## Extensions Overview

Emblem is hackable, that is, arbitrary functionality may be added by its users.
This is done in one of two ways:

1. In code executed as the document goes through its typesetting cycle (‘extensions’), or
2. ~~In code executed to convert to a given output format (‘output drivers’).~~ _Not yet implemented_

### Typesetting-time extensions

Typesetting-time extensions, hereafter referred to simply as ‘extensions,’ are snippets of [Lua][lua] code which are imported after the document has been parsed and are executed as it undergoes its typesetting run.

#### Emblem Public Table

Extensions can define functions to be accessed in the document by editing the Emblem Public Table stored in the `em` variable.
For example, we may wish create a document which includes a note of how long it took to compile.
To do this, we must create a function which creates the desired string, and place it into some field in the `em` table, say `cpu_time_used`

```lua
em.cpu_time_used = function()
	return 'This document was typeset in ' .. os.clock() .. ' seconds of CPU time'
end
```

If this code is written in a file called `cpu-time.lua`, it can be imported by adding the flag `-fcpu-time` when `em` is run (note the file extension is optional).
Now, when the directive `.cpu_time_used` is seen in the document, the above code will be executed, it will be replaced with the message.

#### Evaluation Order

Emblem is lazy, that is, it tries to do no more work than is necessary to typeset a document.
So for example, if a directive takes three inputs and just outputs the first one, emblem will not bother to evaluate the others.
This is because by default, Emblem will only evaluate a node if it can guarantee that it will appear in the output.

Sometimes, however, it can be useful to force Emblem to use a different evaluation order, such as to inspect the results which would not otherwise appear directly in the output.
This can be done using the `eval` function, which takes a node and evaluates it, or the `eval_string` function which
Evaluation order is manipulated in the definition of the `.if` directive, which looks something like the following:

```lua
em.if = function(c, b)
	cs = eval_string(c)
	if cs == '' or cs == '0' or cs == 'false' then
		return b
	else
		return nil
	end
end
```

Here, although input `c` is never present in what is returned, by calling `eval_string` upon it we can reason about it.

#### Useful Functions

The following functions are defined in Emblem’s standard library and are likely useful to extension writers.

| Package     | Function          | Description                                                                                                                                        |
| ----------  | ---------         | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `std.base`  | `is_list`         | Takes a table and outputs true if it represents a list (its keys form a sequence of consecutive integers from 1 to the number of items stored)     |
| `std.base`  | `show`            | Takes an value and generates a string which represents it and its contents                                                                         |
| `std.base`  | `showp`           | Same as `std.base.show` but the string representation is slightly prettier                                                                         |
| `std.base`  | `keys`            | Extracts the set of keys from a table                                                                                                              |
| `std.base`  | `values`          | Extracts the set of values from a table                                                                                                            |
| `std.base`  | `node_string`     | Takes a table which represents a node in the document structure and extracts all its words into a string                                           |
| `std.base`  | `eval_string`     | Takes a reference to a node, evaluates it and calls `std.base.node_string` upon it                                                                 |
| `std.base`  | `elem`            | Takes a value `v` and a list `vs` and returns whether `v` is in `vs`                                                                               |
| `std.base`  | `mkcall`          | Takes a string `s` and returns a function which can be used to construct a document node which represents a call to a directive named `s`          |
| `std.lingo` | `cond`            | Takes a node, evaluates it and returns ‘false’ if its string value is the empty string, `0` or `false` (case-insensitive), otherwise ‘true.’       |
| `std.lingo` | `toint`           | Converts a value to either `1` or `0` depending on its truth                                                                                       |
| `std.lingo` | `get_var`         | Takes a string and returns the value stored within that variable or `nil` if it has not been defined                                               |
| `std.lingo` | `set_var`         | Takes two strings: the name of a variable and a value to assign to it                                                                              |
| `std.lingo` | `undef_var`       | Removes the definition for a given variable                                                                                                        |
| (global)    | `eval`            | Evaluates a node in the document tree and returns a table which represents the result, used force (early) evaluation                               |
| (global)    | `requires_reiter` | Set a flag which marks the document as in need of another typesetting pass                                                                         |

#### Useful Values

The following variables are created by Emblem’s core and are likely useful to extension writers.

| Variable     | Description                                                                                                                                                                  |
| --------     | -----------                                                                                                                                                                  |
| `em`         | Emblem’s public table. Contains a mapping from directive names to functions, checked every time a directive is evaluated to determine whether any extension code must be run |
| `em_iter`    | Holds the current typesetting loop iteration number, starting at 1                                                                                                           |
| `node_types` | Table of node type-names and their associated emblem-core IDs                                                                                                                |

#### Events

To help extensions react to how the document is being processed, there are several events which are triggered.
These are triggered on objects which extend the `Component` class defined in the `std.std` module and are as follows.

1. `on_start`, executed once after all extensions are loaded
2. `on_iter_start`, executed at the start of each iteration
3. `on_iter_end`, executed at the end of each iteration
4. `on_end`, executed once, after the final iteration but before output

There are a number of classes which may be imported from the `std.std` module which provide frameworks for storing data whilst reacting to these events.
For example, the table of contents is a subclass of `Component` which stores the names and numbers of headings as the document is processed, requesting another run if the table of contents at the end of the previous run is different to that at the end of the current (e.g. a page-number has been updated by some other change).

A re-iteration can be requested by calling the `requires_reiter` function in [Lua][lua].
This will cause the typesetting loop to be run again, unless the (user-configurable) number of maximum iterations has been reached.
The number of the current iteration (starting from 1) is accessible through the `em_iter` variable.

#### Useful Classes

The Emblem standard library is written in [Moonscript][moonscript], which compiles to [Lua][lua].
The following concepts are easily described in terms of classes and objects.

| Class           | Subclass of     | Description                                                                                                                                                                                            |
| --------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Component`     | none            | Describes a part of the document which reacts to events, registers itself for updates                                                                                                                  |
| `Counter`       | `Component`     | Represents a number which has its value reset at the start of each typesetting run. Can be incremented, can also automatically be reset in response to another counter being incremented               |
| `SyncContainer` | `Component`     | Represents a container of arbitrary an arbitrary value which, if the value at the end of the current iteration is different to that at the end of the previous, requests another typesetting iteration |
| `SyncList`      | `SyncContainer` | Represents a sync container which holds a list (items ordered, not necessarily unique)                                                                                                                 |
| `SyncSet`       | `SyncContainer` | Represents a sync container which holds a set (items unordered, unique)                                                                                                                                |

### Output Drivers

Emblem is capable of outputting to any format for which it has an output driver.
The binary itself contains some output drivers, but it is also possible to import ones from other sources as desired.

This is currently done through dynamically-linked C libraries.
Due to the logistics of needing to compile these for multiple operating systems, this section of the program will be replaced with a [Lua][lua]-based one in future.

## License and Author

This project is maintained by Ed Jones and is licensed under the GNU General Public License version 3.

## Contributions

Contributions are welcome!

Some notes on the build process.

- The file `em.yml` is used to document the command-line interface and dependencies of the project
- When a new file is added which must be included in the binary (either C or Lua/Moonscript source), `scripts/autogen && ./configure` must be rerun
- `make` will by default compile the `em` binary
- `make check_em` can be used to compile the unit test binary `check_em`
- `make dist` will generate an archive containing all things present in a distributable version of the project
- `make format` can be used to format the code
- `make lint` can be used to run the linter

[bodoni-star]: https://indestructibletype.com/Bodoni.html
[criterion]: https://github.com/Snaipe/Criterion
[css-ref]: https://www.w3schools.com/css/css_intro.asp
[gcc]:
[git]:
[gnu-autotools]:
[lua]: https://www.lua.org
[moonscript]: https://moonscript.org
[releases-page]: https://www.github.com/TheSignPainter98/emblem/releases
[sass-scss-ref]: https://sass-lang.com
[unix-tools]:
[yq]: https://kislyuk.github.io/yq/
