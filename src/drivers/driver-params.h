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

typedef struct OutputDriverInf_s
{
	bool supports_typesetting;
} OutputDriverInf;

typedef struct
{
	char* name;
	OutputDriverInf* inf;
	DriverRunner run;
} InternalDriver;
