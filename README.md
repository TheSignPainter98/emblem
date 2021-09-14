# Emblem

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/TheSignPainter98/emblem)](https://github.com/TheSignPainter98/emblem/releases/latest)
[![Travis (.org)](https://img.shields.io/travis/TheSignPainter98/emblem)](https://app.travis-ci.com/github/TheSignPainter98/emblem)
[![GitHub stars](https://img.shields.io/github/stars/TheSignPainter98/emblem)](https://github.com/TheSignPainter98/emblem/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/TheSignPainter98/emblem)](https://github.com/TheSignPainter98/emblem/issues)
[![GitHub license](https://img.shields.io/github/license/TheSignPainter98/emblem)](https://github.com/TheSignPainter98/emblem/blob/master/LICENSE)

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
		* [Misc Standard directives](#misc-standard-directives)
	* [Preprocessor Directives (‘Pragmas’)](#preprocessor-directives-pragmas)
* [Styling Overview](#styling-overview)
* [Extensions Overview](#extensions-overview)
	* [Typesetting-time extensions](#typesetting-time-extensions)
		* [Emblem Public Table](#emblem-public-table)
		* [Evaluation Order](#evaluation-order)
		* [Useful Functions](#useful-functions)
		* [Useful classes](#useful-classes)
		* [Useful Values](#useful-values)
		* [Events](#events)
	* [Input Drivers](#input-drivers)
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

Emblem also supports extensions which can be added to the document-environment by the `-x[ext]` option.
Extensions can also be passed text values using the `-a[ext].[param]=[val]` option.
For example, `example.em` could be compiled into bbcode using the `asdf` extension with its argument `fdsa` being set to `qwer`:

```bash
em example -Tbb -xasdf -aasdf.fdsa=qwer
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
- Standard UNIX tools
- [gcc][gcc]

To compile and run the `em`, you will need to install the following libraries:

- [criterion](https://github.com/Snaipe/Criterion)
- [glibc](https://www.gnu.org/software/libc/) (Comes with all good GNU/Linux distributions)
- [libcss](https://www.netsurf-browser.org/projects/libcss/)
- [libsass](https://sass-lang.com/libsass)
- [lua](https://www.lua.org/download.html) 5.4
- [lua-argparse](https://luarocks.org/modules/argparse/argparse)
- [lua-lyaml](https://luarocks.org/modules/gvvaughan/lyaml)
- [lua-htmlparser](https://github.com/msva/lua-htmlparser)
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

Syntactic sugar is also available for the cross-referencing and bibliography systems.

```emblem
An anchor can be set down with @name, and makes not of the current label value at this location
This can then be referenced with #name.
```

Headers and other items may set the current label, allowing the user to mark the location of a cross-reference they want, safe in the knowledge that their cross-references handled for them.
The call `@name` expands to `.anchor{name}` and `#name` responds to `.ref{name}`.

Citations can be marked down in a similar manner, as text inside square brackets without any spaces.

```emblem
This is a sentence I’d like to atrribute to someone else by citing [that_someone]

So their name will also appear later in the

.bib
```

### Emblem Script

Sometimes to can be convenient to affect the structure of the document in a small way without needing to write extension code.
This can be achieved by using the scripting directives define in Emblem’s standard library, which create a small shell-like language.
The scripting language contains a minimal set of directives to perform common programming language functionality.

#### Variables

The `.set-var` function sets a variable to a given value.
For example, `.set-var{x}{asdf}` will set variable `x` to the string `asdf`.

The value of a variable can be retrieved with the `.get-var` function.
After the above, `.get-var{x}` will now return `asdf`.

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

The `.case` directive takes a number, _n_, and cases _cs_ (the rest of its inputs), and returns the _n_-th entry in _cs_ if it exists, otherwise the last.

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

#### Misc Standard directives

| Directive           | Description                                                                                                        |
| ------------------- | ---                                                                                                                |
| `.anchor`           | Set down a cross-reference anchor, subsequent references to this anchor will return the label value here.          |
| `.bib`              | Output a bibliography here, optionally specify the source file of the references.                                  |
| `.case`             | Control flow directive, see [above](#control-flow).                                                                |
| `.cite`             | Place a citation marker down and have a corresponding entry appear in the bibliography.                            |
| `.def`              | Define a custom directive.                                                                                         |
| `.defined`          | Returns whether a given variable has been defined in the current scope.                                            |
| `.echo`             | Write a string representation of its inputs `stdout`.                                                              |
| `.echo-on`          | Same as `.echo`, but its first input is a number _n_, and output is only performed on the _n_-th typesetting pass. |
| `.error`            | Output an error and quit.                                                                                          |
| `.error-on`         | Same as `.error` but its first input is a number _n_, and output is only performed on the _n_-th typesetting pass  |
| `.exists`           | Returns whether a directive would execute extension code.                                                          |
| `.foreach`          | Control flow directive, see [above](#control-flow).                                                                |
| `.get-var`          | Returns the value of a given variable in the current context.                                                      |
| `.h1`               | Constructs a level 1 header.                                                                                       |
| `.h2`               | Constructs a level 2 header.                                                                                       |
| `.h3`               | Constructs a level 3 header.                                                                                       |
| `.h4`               | Constructs a level 4 header.                                                                                       |
| `.h5`               | Constructs a level 5 header.                                                                                       |
| `.h6`               | Constructs a level 6 header.                                                                                       |
| `.if`               | Control flow directive, see [above](#control-flow).                                                                |
| `.ifelse`           | Control flow directive, see [above](#control-flow).                                                                |
| `.include`          | Read and include a given file here. Can optionally specify a language to use, or rely on emblem’s detection of the file extension. Caches result by file-name to avoid re-running parsers. |
| `.include*`         | Read and include a given file here. Can optionally specify a language to use, or rely on emblem’s detection of the file extension. Always runs the relevant parser. |
| `.known_directives` | Output a list of known directives.                                                                                                                  |
| `.ref`              | Takes a key and returns the label value where an anchor with the same the same key was set down.
| `.set-var`          | Set a variable with a given string value in the current context.                                                                                                                  |
| `.streq`            | Returns whether the text extracted from two inputs is the same.                                                                                                                    |
| `.toc`              | Outputs a table of contents.                                                                                                                   |
| `.undef`            | Undefines a directive.                                                                                                                   |
| `.warn`             | Same as `.error` but outputs a warning instead.                                                                                                                   |
| `.warn-on`          | Same as `.error-on` but outputs a warning instead.                                                                                                                   |
| `.while`            | Control flow directive, see [above](#control-flow).                                                                |

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
This is done in one of three ways:

1. In code executed as the document goes through its typesetting cycle (‘extensions’)
2. In code executed to convert a given input format to Emblem’s internal structures (‘input drivers’)
3. ~~In code executed to convert to a given output format (‘output drivers’).~~ _Not yet fully implemented_

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

If this code is written in a file called `cpu-time.lua`, it can be imported by adding the flag `-xcpu-time` when `em` is run (note the file extension is optional).
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

| Package         | Function          | Description                                                                                                                                                                                                        |
| --------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `std.ast`       | `mkcall`          | Takes a name of a directive, returns a function which constructs a Call node with that name and the rest of its arguments                                                                                          |
| `std.base`      | `copy_loc`        | Copy a location pointer into a table, allows storage of a location after it has been destroyed.                                                                                                                    |
| `std.base`      | `em_loc`          | Return the location in the source which corresponds to the directive which is currently being evaluated.                                                                                                           |
| `std.base`      | `eval_string`     | Evaluate a node tree and extract a                                                                                                                                                                                 |
| `std.base`      | `eval`            | Evaluates a node in the document tree and returns a table which represents the result, used to force (early) evaluation                                                                                            |
| `std.base`      | `get_var`         | Get the value of a given variable in the current context.                                                                                                                                                          |
| `std.base`      | `include_file`    | Runs the core emblem file parser on a given location and returns the result                                                                                                                                        |
| `std.base`      | `iter_num`        | Return the current iteration number.                                                                                                                                                                               |
| `std.base`      | `node_string`     | Extract the text represented by a document tree (must have been evaluated)                                                                                                                                         |
| `std.base`      | `requires_reiter` | Set a flag which marks the document as in-need of another typesetting pass                                                                                                                                         |
| `std.base`      | `set_var_string`  | Set the value of a variable in the current scope, calling `eval_string` to obtain the value.                                                                                                                       |
| `std.base`      | `set_var`         | Set the value of a variable in the current scope to a given value.                                                                                                                                                 |
| `std.bib`       | `cite_styles`     | Table of known citation styles, values are functions which take a reference and outputs the text to go in a citation’s square brackets                                                                             |
| `std.bib`       | `get_cite_style`  | Returns the name of the current citation style                                                                                                                                                                     |
| `std.bib`       | `set_cite_style`  | Sets the current citation style                                                                                                                                                                                    |
| `std.func`      | `co_to_list`      | Evaluates a coroutine’s yielded values to a list.                                                                                                                                                                  |
| `std.func`      | `co_to_map`       | Evaluates a coroutine’s returned `{key,value}` pairs to a table.                                                                                                                                                   |
| `std.func`      | `do_nothing`      | A function which does nothing.                                                                                                                                                                                     |
| `std.func`      | `filter_list`     | Takes a predicate and a list, returns a list of inputted elements which satisfy the predicate.                                                                                                                     |
| `std.func`      | `filter`          | Takes a predicate and a coroutine, yielding yielded values which satisfy the predicate.                                                                                                                            |
| `std.func`      | `id`              | A function which returns its input.                                                                                                                                                                                |
| `std.func`      | `int`             | A coroutine which yields integers in a non-repeating sequence.                                                                                                                                                     |
| `std.func`      | `key_list`        | Returns a list of keys in a given table.                                                                                                                                                                           |
| `std.func`      | `keys`            | Yields keys yielded by a given coroutine.                                                                                                                                                                          |
| `std.func`      | `kv_pairs`        | Yields the key-value pairs of a table.                                                                                                                                                                             |
| `std.func`      | `map`             | Takes a function and a coroutine, yields the value of the function applied to yielded elements of the coroutine.                                                                                                   |
| `std.func`      | `nat`             | Yields the natural numbers in a non-repeating sequence.                                                                                                                                                            |
| `std.func`      | `seq`             | Yields the integers in a sequence, takes input of `first`, `last` and `step` exactly as a Lua for-loop does (`for i=first,last,step do ... end`)                                                                   |
| `std.func`      | `take`            | Takes a predicate and a coroutine, yields from the coroutine until the predicate no longer holds.                                                                                                                  |
| `std.func`      | `value_list`      | Returns a list of values in a given table                                                                                                                                                                          |
| `std.func`      | `whole`           | Yields the whole numbers in a non-repeating sequence                                                                                                                                                               |
| `std.lingo`     | `cond`            | Evaluates its input, returns `false` if the input if `nil`, `false`, `‘’`, `0` or `‘false’` (case-insensitive), otherwise returns `true`                                                                           |
| `std.lingo`     | `toint`           | Returns a concise representation of a condition.                                                                                                                                                                   |
| `std.log`       | `log_debug_on`    | Take an input _n_ and call the rest of the inputs `...`, calls `std.log.log_debug(...)` only on typesetting iteration _n_.                                                                                         |
| `std.log`       | `log_debug`       | Output a given debugging message if the output verbosity is great enough.                                                                                                                                          |
| `std.log`       | `log_err_at_loc`  | Output a given error message at a specified location.                                                                                                                                                              |
| `std.log`       | `log_err_at_on`   | Call `std.log.log_err_at_loc` but only on a specified typesetting iteration.
| `std.log`       | `log_err_here`    | Output an error message which includes the location of the current directive being evaluated.                                                                                                                      |
| `std.log`       | `log_err_on`      | Take an input _n_ and call the rest of the inputs `...`, calls `std.log.log_err(...)` only on typesetting iteration _n_.                                                                                           |
| `std.log`       | `log_err`         | Output a given error message if the output verbosity is great enough. Then, unconditionally exit.                                                                                                                  |
| `std.log`       | `log_info_on`     | Take an input _n_ and call the rest of the inputs `...`, calls `std.log.log_info(...)` only on typesetting iteration _n_.                                                                                          |
| `std.log`       | `log_info`        | Output a given informational message if the output verbosity is great enough.                                                                                                                                      |
| `std.log`       | `log_warn_at_loc` | Output a given warning message at a specified location.                                                                                                                                                            |
| `std.log`       | `log_warn_at_on`  | Call `std.log.log_warn_at_loc` but only on a specified typesetting iteration.                                                                                                                                      |
| `std.log`       | `log_warn_here`   | Output a warning message which includes the location of the current directive being evaluated.                                                                                                                     |
| `std.log`       | `log_warn_on`     | Take an input _n_ and call the rest of the inputs `...`, calls `std.log.log_warn(...)` only on typesetting iteration _n_.                                                                                          |
| `std.log`       | `log_warn`        | Output a given warning message if the output verbosity is great enough.                                                                                                                                            |
| `std.ref`       | `get_label`       | Return the current label value.                                                                                                                                                                                    |
| `std.ref`       | `set_label`       | Set the current label value (the value returned by a ref to an anchor which was set down after the current call to `get_label` but before the next one, or the end of the current scope).                          |
| `std.util`      | `elem`            | Takes a value `v` and a list of values `vs`, returns true iff `v` is a value in `vs`.                                                                                                                              |
| `std.util`      | `eq`              | Compare the equality of two values (recursively if necessary, respecting the `__eq` metamethod)                                                                                                                    |
| `std.util`      | `extend`          | Takes input of two lists and returns their concatenation (pure)                                                                                                                                                    |
| `std.util`      | `is_list`         | Returns whether a given value represents a list, that is, it is a table whose indices are all numeric and which range from one to the length of the table.                                                         |
| `std.util`      | `non_nil`         | Returns whether its input is not `nil`.                                                                                                                                                                            |
| `std.util`      | `on_iter_wrap`    | Takes a function `f` and returns a function which takes input of a value `n` and a list of arguments `...`, and only calls `f(...)` if the current iteration is equal to the number evaluated from `n`.            |
| `std.util`      | `sorted`          | Sorts a list in-place and returns it.                                                                                                                                                                              |

#### Useful classes

The Emblem standard library is written in [Moonscript][moonscript], which compiles to [Lua][lua].
The following concepts are easily described in terms of classes and objects.

| Package         | Function          | Subclass of | Description                                                                                                                                                                                                        |
| --------------- | ----------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `std.ast`       | `Call`            | `Node`      | Represents a document directive-call node.                                                                                                                                                                         |
| `std.ast`       | `Content`         | `Node`      | Represents a document content node.                                                                                                                                                                                |
| `std.ast`       | `Word`            | `Node`      | Represents a document word node                                                                                                                                                                                    |
| `std.bib`       | `Bib`             | `SyncSet`   | Constructs a bibliography object, optionally takes the name of the header to use.                                                                                                                                  |
| `std.events`    | `Component`       | N/A         | Represents an object which responds to the events: `on_start`, `on_iter_start`, `on_iter_end` and `on_end`.                                                                                                        |
| `std.events`    | `Counter`         | `Component` | Represents an integer which is incremented with each use and is reset at the start of each iteration, or when another counter which lists it as a sub-counter is incremented.                                      |
| `std.events`    | `SyncBox`         | `Component` | A container for a single value, requests a typesetting loop re-run if value at the end of the current iteration is different from that at the end of the previous. Can be passed an initial value, default `0`.    |
| `std.events`    | `SyncContainer`   | `Component` | A container for a compound value, requests a typesetting loop re-run if value at the end of the current iteration is different from that at the end of the previous. Can be passed an initial value, default `{}`. |
| `std.events`    | `SyncList`        | `Component` | A `SyncContainer` which represents a list (ordered elements)                                                                                                                                                       |
| `std.events`    | `SyncMap`         | `Component` | A `SyncContainer` which represents a mapping (indexable key-value pairs).                                                                                                                                          |
| `std.events`    | `SyncSet`         | `Component` | A `SyncContainer` which represents a set (unique elements).                                                                                                                                                        |
| `std.hdr`       | `Toc`             | `SyncList`  | Constructs a table of contents object                                                                                                                                                                              |

#### Useful Values

The following variables are created by Emblem’s core and are likely useful to extension writers.

| Package         | Variable                | Description                                                                                                                                                                                                        |
| --------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `std.base`      | `em`                    | Emblem public table, a map of callables used when evaluating a directive. Checked every time a directive is evaluated to determine whether any extension code must be run.                                         |
| `std.base`      | `vars`                  | List of contexts containing the values of variables.                                                                                                                                                               |
| `std.constants` | `node_flags`            | Flags which may be bitwise-or’d together, and are accepted by the core.                                                                                                                                            |
| `std.constants` | `node_types`            | IDs of the types of nodes recognised when the core unpacks table-representations of syntax trees. The classes in `std.ast` construct the such objects.                                                             |
| `std.lingo`     | `known_languages`       | Map of languages to their respective interpreters for use with `.include` directives.                                                                                                                              |
| `std.lingo`     | `known_file_extensions` | Map of file extensions to their respective languages for use with `.include` directives.                                                                                                                           |
| `std.style`     | `stylers`               | A map of common styling functions such as italic and bold                                                                                                                                                          |

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

### Input Drivers

Emblem can take input of any format for which it has an input driver.
When Emblem inputs a file through the `.include` directive, a language can optionally be specified in the second parameter to determine the parser to use.
This language is used to look up the parser to use in the `std.lingo.known_languages` table.
An input driver is simply a parser function which has been added into this table (possibly by some extension).

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
[css-ref]: https://www.w3schools.com/css/css_intro.asp
[gcc]: https://gcc.gnu.org
[git]: https://git-scm.com
[gnu-autotools]: https://www.gnu.org/software/automake/manual/html_node/Autotools-Introduction.html
[lua]: https://www.lua.org
[moonscript]: https://moonscript.org
[releases-page]: https://www.github.com/TheSignPainter98/emblem/releases
[sass-scss-ref]: https://sass-lang.com
[yq]: https://kislyuk.github.io/yq/
