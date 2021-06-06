#pragma once

#include "data/str.h"
#include "doc-struct/ast.h"
#include <stdbool.h>

typedef struct
{
	Str* output_stem;
} DriverParams;

typedef int (*DriverRunner)(Doc* doc, DriverParams* params);

struct OutputDriverInf_s;

typedef enum
{
	INTERNAL,
	EXTERNAL,
} DriverType;

typedef struct
{
	DriverType type;
	struct OutputDriverInf_s* inf;
	void* lib_handle;
	Str* driver_name;
	Str* driver_lib_name;
	DriverRunner run;
} OutputDriver;

#define TS_NONE			 0x0
#define TS_BASIC_STYLING 0x1
#define TS_COLOUR		 0x2
#define TS_IMAGE		 0x4
#define TS_PLACEMENT	 0x8
#define TS_SVG			 0x10
typedef int TypesettingSupport;

typedef struct OutputDriverInf_s
{
	TypesettingSupport support;
} OutputDriverInf;

typedef struct
{
	char* name;
	OutputDriverInf* inf;
	DriverRunner run;
} InternalDriver;
