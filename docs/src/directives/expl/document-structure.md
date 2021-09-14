# Document structure

When writing large documents, it is typically necessary to convey the structure of the piece to the reader to allow them to not only focus their attention where is most beneficial to them, but also to convey the significance of all parts of the text.

For this, it is customary to use a system of headings---declarations of upcoming content---and a table of contents to allow a reader to navigate to wherever they will.

An analogous structure may be befitting of an author---to avoid an overlarge monolithic source, Emblem provides directives and pragmas to allow the source of a document to be partitioned into many different files.

In this section, we will detail each of these concepts:

- [Sectioning documents with headers](../h1-6.md)
- [Creating a table of contents](../toc.md)
- [Breaking document source into multiple files](../include.md)

## Example - Multi-section document

The following is a possible outline of the sections of a technical report.

```java
# Introduction
# Background
## Motivating Example
## Concepts
# Literature Review
# Design
## Specification
## Architecture
## UI
# Implementation
## Tools and Libraries
## User-Extensions
# Results and Discussion
# Further Analysis and Extensions
# Evaluation
# Conclusion
```

## Example - Multi-file document

If the above example were written in a single file, it would be reasonable to assume that editing the document would slowly become more and more cumbersome.
A more manageable structure may result from taking each heading with both subheadings and a significant amount of content beneath it, and place this in a directory beneath the project root.

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

As we are only including Emblem source files, we can make use of the `:include` pragma to invoke the source of each document, hence the main file, `report.em`---and also the main file in each of the sections in subdirectories will look something like the below.

As a part of the file inclusion process for Emblem source, if a file `file.em` is not found, then `file/file.em` is tried, hence the root does not need to encode exactly how its contained sections are laid out in the file system.

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

Where possible, the user should try to use the `:include` pragma instead of the `.include` directive as the former is more efficient, skipping a translation step between extension space and the core, which contains the Emblem parser used by both methods.
