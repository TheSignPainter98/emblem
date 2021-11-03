/**
 * @file style.h
 * @brief Exposes function for importing stylesheets from extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "doc-struct/ast.h"
#include "ext-env.h"
#include <stdbool.h>

#define STYLER_LP_LOC "_em_styler"

void provide_styler(ExtensionEnv* e);

void set_ext_style_globals(ExtensionState* s);
int import_stylesheets_from_extensions(ExtensionState* s, Styler* styler, bool parse_css);

int pack_style(ExtensionState* s, Style* style, DocTreeNode* node);
