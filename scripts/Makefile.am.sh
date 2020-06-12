#!/bin/bash

srcs=$(echo ./src/argp.c ./src/argp.h $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$') | sed 's/\//\\\//g')
tests=$(echo ./src/argp.c ./src/argp.h $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -v 'em.c$' | grep -v 'em.h$') $(find check -name '*.c' -or -name '*.h') | sed 's/\//\\\//g')

echo $srcs
(sed "s/<SRC_FILES>/$srcs/" \
	| sed "s/<TEST_FILES>/$tests/"\
	) < /dev/stdin > /dev/stdout
