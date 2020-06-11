#!/usr/bin/automake

# Required by libtoolize
ACLOCAL_AMFLAGS = -I m4

# # Use a library
# lib_LTLIBRARIES = libspkg.la
# libspkg_la_SOURCES = src/spkg/spkg.c src/spkg/spkg.h

# Binary and sources
bin_PROGRAMS = hello_world
# BUILT_SOURCES = y.tab.h
hello_world_SOURCES = src/argp.c src/em.c src/measurement/measurement.c src/doc-struct/doc-ast.c src/logs/logs.c
# hello_world_LDADD = -lspkg
# src/bibtex_parser.y src/bibtex_lexer.l
hello_world_CFLAGS = -Wall -Wextra -Wpedantic -Werror -pedantic-errors -std=gnu18
hello_world_LFLAGS = --bison-bridge --bison-locations -sL -R
hello_world_YFLAGS = -Wall -Wdangling-alias -Wno-yacc -Werror -LC --locations -l -d

# Add documentation
dist_doc_DATA = README.md LICENSE ChangeLog

ChangeLog: scripts/changelog.sh
	. $< > $@

# # Add tests
# TESTS = check_hello_world
# check_PROGRAMS = check_hello_world
# check_hello_world_SOURCES = test/test.c src/hello_world.c
# check_hello_world_CFLAGS = -Wall -Wextra -Wpedantic -Werror -pedantic-errors -std=gnu18
# check_hello_world_LDADD = -lcriterion

y.tab.h: src/hello_world-bibtex_parser.o
