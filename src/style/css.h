/**
 * @file css.h
 * @brief Exposes functions to handle styling and stylesheets
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "css-params.h"
#include "data/str.h"

void make_styler(Styler* styler, Args* args) __attribute__((nonnull(1, 2)));
void dest_styler(Styler* styler) __attribute__((nonnull(1)));
int prepare_styler(Styler* styler) __attribute__((nonnull(1)));

void make_style(Style* style) __attribute__((nonnull(1)));
void dest_style(Style* style) __attribute__((nonnull(1)));
void make_style_data(StyleData* style, Str* name_name) __attribute__((nonnull(1, 2)));
void dest_style_data(StyleData* style) __attribute__((nonnull(1)));

int append_style_sheet(Styler* styler, Str* sheet_loc) __attribute__((nonnull(1)));
