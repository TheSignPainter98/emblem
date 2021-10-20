#pragma once

#include "argp.h"
#include "css-params.h"
#include "doc-struct/ast.h"
#include <libcss/libcss.h>

int compute_style(Styler* s, DocTreeNode* node) __attribute__((nonnull(1, 2)));

int make_style_selection_engine(StyleSelectionEngine* engine);
int dest_style_selection_engine(StyleSelectionEngine* engine);

void make_stylesheet_params(css_stylesheet_params* params, Args* args);
void dest_stylesheet_params(StylesheetParams* params);
