#!/bin/bash

extension_lib_srcs=($(find src/ext/lib/ -name '*.moon'))
extension_lib_built_srcs=($(find src/ext/lib/ -name '*.moon' | for f in $(</dev/stdin); do echo ${f%.*}.lc; done))
built_srcs=(./src/argp.c ./src/argp.h ./src/pp/ignore_warning.h ${extension_lib_built_srcs[@]})
parser_srcs=(src/parser/emblem-lexer.l src/parser/emblem-parser.y)
srcs=(${built_srcs[@]} ${parser_srcs[@]} $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -Pv 'emblem-(lexer|parser).[ch]$' | grep -v 'src/pp/ignore_warning.h$'))
tests=(${built_srcs[@]} ${parser_srcs[@]} $(find src -name '*.c' -or -name '*.h' | grep -v 'argp.c$' | grep -v 'argp.h$' | grep -Pv 'emblem-(lexer|parser).[ch]$' | grep -v 'em.c$' | grep -v 'em.h$') $(find check -name '*.c' -or -name '*.h'))
scripts=($(find scripts -type f | grep -v '.*\.swp'))
dist_data=($(find share/emblem/ -type f))

deps_cflags=$(yq -y '.deps | map("$(" + .name + "_CFLAGS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')
deps_libs=$(yq -y '.deps | map("$(" + .name + "_LIBS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')
check_deps_cflags=$(yq -y '.check_deps | map("$(" + .name + "_CFLAGS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')
check_deps_libs=$(yq -y '.check_deps | map("$(" + .name + "_LIBS)")' em.yml | cut -d' ' -f2- | tr '\n' ' ' | sed 's/ $//')

lintable_srcs=($(./scripts/lintable-srcs.sh))

function pofile()
{
	f=$(basename $1)
	d=$(dirname $1)
	e=${f#*.}
	fb=${f%.*}
	[[ $d =~ src/ext/lib ]] && return
	if [[ $e =~ ^[ly]$ ]]; then
		echo $d/.deps/em-em-$fb.Po
	else
		echo $d/.deps/em-$fb.Po
	fi
}

extra_dist=(${scripts[@]} ${extension_lib_srcs[@]})
source_dependency_files=($(for s in ${srcs[@]}; do [[ "${s##*.}" =~ ^[cly]$ ]] && pofile $s; done))
extra_dist=(${scripts[@]} ${extension_lib_srcs[@]} ${source_dependency_files[@]})

m4 -PE - Makefile.am.in > Makefile.am << EOF
m4_define(S_SRC_FILES, ${srcs[@]})m4_dnl
m4_define(S_TEST_FILES, ${tests[@]})m4_dnl
m4_define(S_BUILT_SRCS, ${built_srcs[@]})m4_dnl
m4_define(S_DEPS_CFLAGS, $deps_cflags)m4_dnl
m4_define(S_DEPS_LIBS, $deps_libs)m4_dnl
m4_define(S_CHECK_DEPS_CFLAGS, $check_deps_cflags)m4_dnl
m4_define(S_CHECK_DEPS_LIBS, $deps_libs $check_deps_libs)m4_dnl
m4_define(S_DIST_DATA, ${dist_data[@]})m4_dnl
m4_define(S_LINTABLE_SRCS, ${lintable_srcs[@]})m4_dnl
m4_define(S_EXTRA_DIST, ${extra_dist[@]})m4_dnl
EOF
