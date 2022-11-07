/**
 * @file preprocess-css.h
 * @brief Exposes functions to call a preprocessor on CSS documents
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "config.h"
#include "css-params.h"
#include "data/str.h"

int preprocess_css(char** data_out, Str* loc, Str* data, StylePreprocessorParams* params) __attribute__((nonnull(1, 2, 4)));
