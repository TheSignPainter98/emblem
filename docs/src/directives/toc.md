# `.toc`

The `.toc` directive creates a table of contents by making a list of all of the headings of a document.
As this list is only complete at the end of a run, after the `.toc` has been processed, a document with a table of contents always requires at least two iterations of the typesetting loop.
No arguments are taken.

```emblem
.toc
```

If there are headings which an author does not wish to be seen in the contents table, starred variants of the heading directives should be used, that is, every appearance of `.h1` or `# ...` should be replaced with `.h1*` or `#* ...` respectively, for `.h1` to `.h6`, and `# ...` to `###### ...`.
