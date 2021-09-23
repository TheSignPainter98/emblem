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

typedef enum
{
	UNPROCESSED = 1,
	PROCESSED	= 0,
	FAIL		= -1,
} CssPreprocessResult;

CssPreprocessResult preprocess_css(char** data_out, char* data_in, Str* fname, StylePreprocessorParams* params);
