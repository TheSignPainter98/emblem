#pragma once

#include "data/str.h"
#include "ext/ext-env.h"

typedef struct
{
	int first_line;
	int first_column;
	int last_line;
	int last_column;
	Str* src_file;
} Location;

Location* dup_loc(Location* todup);
void set_ext_location_globals(ExtensionState* s);
