#pragma once

#include "data/list.h"
#include "doc-struct/ast.h"
#include "ext-params.h"

int ext_eval_tree(ExtensionState* s);
int get_ast_type_name(ExtensionState* s);

int unpack_lua_result(DocTreeNode** result, ExtensionState* s, DocTreeNode* parentNode);
