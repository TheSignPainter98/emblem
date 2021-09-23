# `.curr-version`

When developing documents and iterating through multiple cycles of drafting and improvements, it is possible that multiple versions of the same document may be in close proximity, thus leading to some confusion.
This can be avoided by adding some indication of the document’s version in the output.

The `.curr-version` directive does this my making use of the between-run store.

## Example -- Placing the version in the title

```emblem
# Research, Re: Search and Re-Search (version .curr-version)
```
