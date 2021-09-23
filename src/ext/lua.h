/**
 * @file lua.c
 * @brief Provides functions for executing evaluation-passes on document trees
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "doc-struct/ast.h"
#include "ext-env.h"

#define EM_LOC_NAME "em_loc"

void inc_iter_num(Doc* doc);

int exec_lua_pass(Doc* doc);
int exec_lua_pass_on_node(ExtensionState* s, DocTreeNode* node, int curr_iter);

int to_userdata_pointer(void** val, ExtensionState* s, int idx, LuaPointerType type);
