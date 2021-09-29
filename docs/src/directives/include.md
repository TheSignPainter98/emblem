# `.include` and `.include*`

When writing large documents which consist of a large number of sections, it can be beneficial to partition the source into a number of smaller files.
The result can be a document which is much easier to work upon than a single monolithic source file.

In Emblem, there are three methods by which the contents of another file can be included in a document:

1. The `.include` directive,
2. The `.include*` directive and
3. The `:include` pragma (notice the ‘:’ prefix rather than ‘.’).

We will explain the [basic usage](#basic-usage) of each, before discussing how [paths](#file-name-handling) and [languages](#languages-and-input-drivers) are handled.

## Basic Usage

In basic use, the `.include` directive takes input of a file name with either no extension or ending in ‘.em,’ tries to find that file, parses it as an Emblem document and then returns the result.
This result is also cached to avoid repeatedly re-parsing the same file.

The contents of the specified file is then pasted into the document in the current location.
Due to how paragraphs are recognised, if the call to `.include` is the only thing in a paragraph, then its contents are scanned to discern paragraphs, however, if it is _not_ the sole member of the paragraph, then its contents is added directly into the paragraph.

```emblem
.include: some-file // Paragraphs in some-file.em are parsed as if were written directly in the source file

Some sentence.
.include: some-file // Content of some-file.em are pasted into the surrounding paragraph so likely appear as more sentences.
Some other sentence.
```

If the file being read is expected to change between typesetting iterations, the `.include*` directive may be used, which does not cache its result.
This is hence less efficient for un-changing source files.
The `.include*` directive shares all other properties of the `.include` directive.

The `:include` _pragma_ differs from the `.include` directive as it is handled at parse-time, beyond the extension environment and as such, the file-path is fixed.
The following will include ‘some-file.em’ like the above---

```emblem
:include "some-file"
```

As this is handled separately from extensions, the file path given to `:include` is constant---it cannot be changed dynamically by the user and is read exactly as seen in the source file.
Hence although the `:include` pragma is restrictive, it provides a guarantee to external tools parsing Emblem source as to the structure of the document, without needing to implement evaluation mechanics.

## File name handling

The sections in a large document naturally differ in size---the introduction and conclusion are likely dwarfed by the body of a report, the preamble and acknowledgements are likely dwarfed by these in turn, and even among the main passages of work, some will bring together content from many sources and some only few.
As such, it can be useful to partition a document’s source files into multiple folders, each helping group together relevant materials.
The file structure of a report may hence look something like this, with `report.em` as the main file.

```tree
.
|- report.em
|- introduction.em
|- background/
|   |- background.em
|   |- motivating-example.em
|   |- concepts.em
|- literature-review.em
|- design/
|   |- design.em
|   |- specification.em
|   |- architecture.em
|   |- ui.em
|- implementation/
|   |- implementation.em
|   |- tools-and-libraries.em
|- results-and-discussion.em
|- further-analysis-and-extensions.em
|- evaluation.em
|- conclusion.em
```

In Emblem, this file structure is embraced by the method by which Emblem searches for ‘.em’ files.
When given a file name either `file.em` or just `file` (no extension), Emblem first searches for that file in the current directory, and if this is not found, it searches for `file/file.em`.
As such, we can include all of the required ‘.em’ files from the main one _without_ needing to know how the files are laid out on disk, allowing for a more abstracted view.
The main file can hence be written as:

```emblem
:include "introduction"
:include "background"
:include "literature-review"
:include "design"
:include "implementation"
:include "results-and-discussion"
:include "further-analysis-and-extensions"
:include "evaluation"
:include "conclusion"
```

## Languages and input drivers

So far, we have only discussed the inclusion of Emblem source files, however, this is not the only format accepted by Emblem.
By making use of input drivers either _explicitly_ or _implicitly,_ we can make use of document source written in other languages.

To explicitly make use of input drivers, we can make use of the optional second argument of the `.include` and `.include*` directives.
In this parameter, we can specify the language of the given source file, prompting Emblem to look for a parse function associated with that language.[^input-drivers]
The result of calling this function on the input file is then returned.

```emblem
.include{some-file.html}{html} // Asserts that some-file.html should be parsed as html
```

The implicit use of input drivers helps eliminate some redundancy in the above statement---when a file extension is given, Emblem uses a mapping of extensions to input languages before finding the parse function as before.
As such, the second parameter in the above `.include` call can be removed, relying on Emblem to recognise the input language, hence leading to a cleaner source.

```emblem
.include{some-file.html}
```

As extensions are involved in the processing of languages, we can see that the above logic only applies to the `.include` and `.include*` directives, and not the `:include` pragma.

[^input-drivers]: Recall that input drivers are just extensions which declare an association between an input language and a parse function.
