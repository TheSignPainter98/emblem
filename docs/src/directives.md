# Document directives

In Emblem, the semantic structure of a document is partitioned by the use of directives.
As the document is processed, they are used to enact not only styling changes but also extension functions.

- If an extension function exists a same-name entry in the Emblem public table (`em` to extension-writers), then it is executed
- If there is a style with the same name as the directive, then that is applied.

All syntactic sugars (such as surrounding a word in underscores to make it italic) are simple translations to directive calls.

## Example - Styling part of a sentence

The following lines will all yield the same result---an entire or part of a sentence emboldened by use of the `.bf` directive.

```emblem
.bf: This entire sentence is bold
This sentence is only has .bf{two bold} words.
Hello **rather obnoxious** world!
```

## Example - Cross-referencing

In academic papers and technical documents, it is often necessary to refer the reader to different sections, often to inform them of the location if pertinent information.
In Emblem, this is done through the label-anchor-reference system: headings and other document items set a label, a document-writer places an anchor at an important location which makes note of the label value there, and then calls a reference to that anchor which returns the required label value.

```emblem
# If this is some section with some useful things in (behind the scenes it edits the current label value, for example this may be section 6.2.3)

And perhaps this is the location of that useful thing, then an anchor is set with .anchor{a-useful-thing} or the syntactic sugar @useful-thing.

# Some other document sections may then be written

Which contain other things to interest the reader.

When the time comes, and the reader’s attention must be brought back to what was before, the relevant label can be automatically determined when by calling .ref{a-useful-thing} or #a-useful-thing.
```
