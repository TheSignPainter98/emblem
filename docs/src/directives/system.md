# `.$`

Emblem allows the user to run arbitrary shell commands with use of the `.$` or ‘system’ directive.
This takes input of a command, and optionally text to pipe into the standard input of that command.
As these are run externally, this leads to two problems:

1. The command given to a system directive has no guarantee of portability
2. This directive has all the security risks associated with random code execution so **be careful when executing someone else’s source code**.
	This can be alleviated by use of [sandboxing][sandboxing].

The system directive is given the `.$` name for visual similarity to a terminal when used.

## Example -- Listing files in the current directory

Using the standard UNIX [`ls`][ls] program, we can see the contents of the current directory:

```emblem
.$: ls
```

## Example -- Counting the number of unique words in a passage of text

As the optional second argument to `.$` is text to pass into a pipe, it is possible to encode all interactions with a sub-process in an Emblem document.

The following pipeline will move all words in the supplied text into individual lines ([`tr`][tr]), remove almost all non-alphabetic characters such as punctuation ([`grep`][grep]), performs a [`sort`][sort] on them, filter the resultant list so it only contains unique entries ([`uniq`][uniq]) before finally performing a word-count ([`wc`][wc]).

```emblem
.${tr " " '\\n' | grep -o '[a-zA-Z-]' | sort | uniq -i | wc -w}:
	Hello, this is some text which may contain repeated words.
	We wonder how long it is, but we we’d rather know now many repeated words the text might contain.
```

[sandboxing]: ../generated/command-line-args.md#-s---sandbox-level
[wc]: https://www.wikiwand.com/en/Wc_(Unix)
[ls]: https://www.wikiwand.com/en/Ls
[grep]: https://www.wikiwand.com/en/Grep
[sort]: https://www.wikiwand.com/en/Sort_(Unix)
[uniq]: https://www.wikiwand.com/en/Uniq
[tr]: https://www.wikiwand.com/en/Tr_(Unix)
