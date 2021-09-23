/**
 * @file lua-lib-load.h
 * @brief Exposes function to load standard Lua libraries into extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "doc-struct/ast.h"

int load_em_std_lib(ExtensionState* s);
