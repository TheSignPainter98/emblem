/**
 * @file lua.c
 * @brief Provides functions for executing evaluation-passes on document trees
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "doc-struct/ast.h"
#include "ext-env.h"
#include <stdbool.h>

#define EM_LOC_NAME "em_loc"

void inc_iter_num(Doc* doc);

int exec_ext_pass(Doc* doc);
int exec_ext_pass_on_node(ExtensionState* s, Styler* sty, DocTreeNode* node, int curr_iter, bool foster_paragraphs) __attribute__((nonnull(1, 2)));
