#!/bin/bash

libs=($(find src/ext/lib/ -type f -name '*.moon' | grep -v '__mod' | xargs ./scripts/moon-po-lin | sed 's/.moon$/.lc/'))
extension_lib_loaders=$(for f in ${libs[@]}; do echo "#include \"${f#src/ext/}\""; done)

cat << EOF
/**
* @file lua-lib-load.c.sh
* @brief Implements the standard Lua library loader function
* @author Edward Jones
* @date 2021-09-17
*/
#include "lua-lib-load.h"

#include "ext-params.h"
#include "logs/logs.h"
#include <lauxlib.h>

/**
* @brief Load standard Lua library into extension space
*
* @param s Extension state into which to load the libraries
*
* @return \`0\` iff successful
*/
int load_em_std_lib(ExtensionState* s)
{
$extension_lib_loaders
	return 0;
}
EOF
