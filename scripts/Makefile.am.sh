#!/bin/bash

extension_lib_srcs=(`find src/ext/lib/ -name '*.moon' | for f in $(</dev/stdin); do echo ${f%.*}.lc; done`)
built_srcs=(./src/argp.c ./src/argp.h ./src/pp/ignore_warning.h ${extension_lib_srcs[@]})
sanitised_built_srcs=$(echo ${built_srcs[@]} | sed 's/\//\\\//g')
srcs=$(echo ${built_srcs[@]} $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -v 'src/pp/ignore_warning.h$') | sed 's/\//\\\//g')
tests=$(echo ./src/argp.c ./src/argp.h $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -v 'em.c$' | grep -v 'em.h$') $(find check -name '*.c' -or -name '*.h') | sed 's/\//\\\//g')

deps_cflags=$(yq -y '.deps | map("\\$(" + .name + "_CFLAGS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')
deps_libs=$(yq -y '.deps | map("\\$(" + .name + "_LIBS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')
check_deps_cflags=$(yq -y '.check_deps | map("\\$(" + .name + "_CFLAGS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')
check_deps_libs=$(yq -y '.check_deps | map("\\$(" + .name + "_LIBS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')

(sed "s/<SRC_FILES>/$srcs/" \
	| sed "s/<TEST_FILES>/$tests/"\
	| sed "s/<BUILT_SRCS>/$sanitised_built_srcs/"\
	| sed "s/<DEPS_CFLAGS>/$deps_cflags/"\
	| sed "s/<DEPS_LIBS>/$deps_libs/"\
	| sed "s/<CHECK_DEPS_CFLAGS>/$check_deps_cflags/"\
	| sed "s/<CHECK_DEPS_LIBS>/$check_deps_libs/"\
	) < /dev/stdin > /dev/stdout
