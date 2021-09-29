# `.bib` and `.cite`

To reference other works outside of a document, we can use citations which point to bibliography entries with information on how to find the exact source.
In Emblem, these are done using the `.cite` and `.bib` directives.

The `.cite` directive takes input of a _key,_ a unique string which provides a reference to a particular bibliography item.

The bibliography is both parsed and produced by the `.bib` directive.
// TODO: talk about the optional argument!

```emblem
.bib // Reads from the default location
.bib: file // Reads from some-file if it exists, otherwise, file.yml, file.yaml or file.json
```

Unless the optional bibliography file-name parameter is given, this directive reads searches for a file called ‘bib’.


```yaml
carlson2011you:
  title: "You probably think this paper's about you: Narcissists' perceptions of their personality and reputation."
  author: Carlson, Erika N and Vazire, Simine and Oltmanns, Thomas F
  journal: Journal of personality and social psychology
  volume: 101
  number: 1
  pages: 185
  year: 2011
  publisher: American Psychological Association
greenfield2015asdf:
  title: "ASDF: A new data format for astronomy"
  author: Greenfield, P and Droettboom, M and Bray, E
  journal: Astronomy and Computing
  volume: 12
  pages: 240--251
  year: 2015
  publisher: Elsevier
```
