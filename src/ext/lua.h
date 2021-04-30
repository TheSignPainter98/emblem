#pragma once

#include "doc-struct/ast.h"
#include "ext-params.h"

int make_doc_ext_state(Doc* doc, ExtParams* params);
void dest_doc_ext_state(Doc* doc);

void inc_iter_num(ExtensionEnv* e);

int exec_lua_pass(Doc* doc);
int exec_lua_pass_on_node(ExtensionState* s, DocTreeNode* node);
