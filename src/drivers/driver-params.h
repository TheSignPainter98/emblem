/**
 * @file driver-params.h
 * @brief Exposes functionality to handle output drivers parameters
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "data/str.h"
#include "doc-struct/ast.h"
#include "ext/ext-env.h"
#include <stdbool.h>

struct OutputDriver_s;

typedef int (*DriverRunner)(struct OutputDriver_s* driver, Doc* doc, ExtensionEnv* ext, Str* time_str);

typedef int TypesettingSupport;
#define TS_NONE			 (1 << 0)
#define TS_BASIC_STYLING (1 << 1)
#define TS_COLOUR		 (1 << 2)
#define TS_IMAGE		 (1 << 3)
#define TS_TEXT_SIZE	 (1 << 4)
#define TS_PLACEMENT	 (1 << 5)
#define TS_SVG			 (1 << 6)

typedef struct OutputDriver_s
{
	TypesettingSupport support;
	bool use_stdout;
	Str* output_stem;
	bool requires_stylesheet;
	DriverRunner run;
} OutputDriver;

typedef struct
{
	char* name;
	TypesettingSupport support;
	bool requires_stylesheet;
	DriverRunner run;
} InternalDriver;
