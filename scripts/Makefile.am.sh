#!/bin/bash

built_srcs=(./src/argp.c ./src/argp.h ./src/pp/ignore_warning.h)
sanitised_built_srcs=$(echo ${built_srcs[@]} | sed 's/\//\\\//g')
srcs=$(echo ${built_srcs[@]} $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -v 'src/pp/ignore_warning.h$') | sed 's/\//\\\//g')
tests=$(echo ./src/argp.c ./src/argp.h $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -v 'em.c$' | grep -v 'em.h$') $(find check -name '*.c' -or -name '*.h') | sed 's/\//\\\//g')

(sed "s/<SRC_FILES>/$srcs/" \
	| sed "s/<TEST_FILES>/$tests/"\
	| sed "s/<BUILT_SRCS>/$sanitised_built_srcs/"\
	) < /dev/stdin > /dev/stdout
