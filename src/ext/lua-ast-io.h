/**
 * @file lua-ast-io.h
 * @brief Exposes functions for translating between Lua tables and document trees
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "data/list.h"
#include "doc-struct/ast.h"
#include "ext-params.h"

int unpack_ext_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode);
void register_ext_node(ExtensionState* s);
