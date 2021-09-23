# C API

Following the C-style of headers and body files, this section of the docs follows the same partition between [module interfaces][headers], which correspond to files `*.h` in the source repo, and [module bodies][bodies], which correspond to files `*.c`.
It must be noted that some of the files in this section of the docs do not directly exist in the source as they are auto-generated.
Some examples of this are the [lexer body][lexer-body] and [parser body][parser-body], which is are generated from [Flex][flex] and [Bison][bison] sources respectively.

[bison]: https://www.wikiwand.com/en/GNU_Bison
[bodies]: ./internal-api-module-interfaces.md
[flex]: https://www.wikiwand.com/en/Flex_(lexical_analyser_generator)
[headers]: ./internal-api-module-internals.md
[lexer-body]: ./generated/parser/emblem-lexer.h.md
[parser-body]: ./generated/parser/emblem-parser.h.md
