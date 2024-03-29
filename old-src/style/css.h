/**
 * @file css.h
 * @brief Exposes functions to handle styling and stylesheets
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "css-params.h"
#include "data/str.h"
#include "ext/ext-env.h"
#include <stdbool.h>

void make_styler(Styler* styler, Args* args) __attribute__((nonnull(1, 2)));
void dest_styler(Styler* styler) __attribute__((nonnull(1)));
int prepare_styler(Styler* styler, ExtensionState* s) __attribute__((nonnull(1, 2)));

void make_style(Style* style) __attribute__((nonnull(1)));
void dest_style(Style* style) __attribute__((nonnull(1)));
void make_style_data(StyleData* style, Str* name_name, struct DocTreeNode_s* node) __attribute__((nonnull(1, 2, 3)));
void dest_style_data(StyleData* style) __attribute__((nonnull(1)));

int append_style_sheet(Styler* styler, Str* sheet_loc, Str* sheet_data, bool append_css_to_context) __attribute__((nonnull(1, 2)));
