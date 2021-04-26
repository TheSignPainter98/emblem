#pragma once

#include "css-params.h"
#include "data/str.h"
#include "doc-struct/ast.h"

void make_styler(Styler* styler, Args* args) __attribute__((nonnull(1, 2)));
void dest_styler(Styler* styler) __attribute__((nonnull(1)));
int prepare_styler(Styler* styler) __attribute__((nonnull(1)));

void dest_style(Style* style) __attribute__((nonnull(1)));

int append_style_sheet(Styler* styler, Str* sheet_loc) __attribute__((nonnull(1)));
