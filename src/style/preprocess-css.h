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
