#!/bin/bash

echo ./src/pp/ignore_warning.h ./src/parser/emblem-parser.h ./src/argp.c $(find src -name '*.c')
find src/ext/lib -name '*.moon' | grep -v 'base.moon$' | tr '\n' ' '
echo
