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

int ext_eval_tree(ExtensionState* s);
int get_ast_type_name(ExtensionState* s);

int pack_tree(ExtensionState* s, DocTreeNode* node);
int unpack_lua_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode);
