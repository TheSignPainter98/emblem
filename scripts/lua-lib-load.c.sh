#!/bin/bash

libs=($(find src/ext/lib/ -type f -name '*.moon' | sed 's/.moon$/.lc/'))
extension_lib_loaders=$(for f in ${libs[@]}; do echo "#include \"${f#src/ext/}\""; done)

cat << EOF
#include "lua-lib-load.h"

#include "ext-params.h"
#include "logs/logs.h"
#include <lauxlib.h>

int load_em_std_lib(ExtensionState* s)
{
	int rc = 0;
$extension_lib_loaders
	return rc;
}
EOF
