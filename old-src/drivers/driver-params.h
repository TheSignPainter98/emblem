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
#include <stdint.h>

struct OutputDriver_s;

typedef int (*DriverRunner)(struct OutputDriver_s* driver, Doc* doc, ExtensionEnv* ext, Str* time_str);

typedef uint_least8_t TypesettingSupport;
#define TS_NONE				   0x0
#define TS_CSS_UNPARSED		   (1 << 0)
#define TS_CSS_STYLES		   (1 << 1)
#define TS_CSS_STYLES_COMPOSED (1 << 2)
#define TS_PLACEMENT		   (1 << 3)
#define TS_PLACEMENT_MONOSPACE (1 << 4)

typedef struct OutputDriver_s
{
	TypesettingSupport support;
	bool use_stdout;
	Str* output_stem;
	DriverRunner run;
} OutputDriver;

typedef struct
{
	char* name;
	TypesettingSupport support;
	DriverRunner run;
} InternalDriver;
