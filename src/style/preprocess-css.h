#pragma once

#include "config.h"
#include "data/str.h"
#include "css-params.h"

typedef enum
{
	UNPROCESSED = 1,
	PROCESSED = 0,
	FAIL = -1,
} CssPreprocessResult;

CssPreprocessResult preprocess_css(char** data_out, char* data_in, Str* fname, StylePreprocessorParams* params);
