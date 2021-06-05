#pragma once

#include "doc-struct/ast.h"
#include "ext-env.h"

void inc_iter_num(Doc* doc);

int exec_lua_pass(Doc* doc);
int exec_lua_pass_on_node(ExtensionState* s, DocTreeNode* node);
