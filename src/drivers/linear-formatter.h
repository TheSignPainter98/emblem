#pragma once

#include "data/list.h"
#include "data/map.h"
#include "data/str.h"
#include "driver-params.h"
#include <stdbool.h>

typedef struct
{
	List* content;
	List* formatter_content;
	Str* output_name_fmt;
	Str* output_name_stem;
	Str* stylesheet_name;
	Map* call_name_map;
} LinearFormatter;

void make_linear_formatter(LinearFormatter* formatter, DriverParams* params, size_t num_special_functions,
	const Pair special_functions[num_special_functions], Str* output_name_fmt);
void dest_linear_formatter(LinearFormatter* formatter);

void concat_linear_formatter_content(LinearFormatter* formatter, List* list);
void append_linear_formatter_raw(LinearFormatter* formatter, char* raw);
void append_linear_formatter_str(LinearFormatter* formatter, Str* str);
void append_linear_formatter_strf(LinearFormatter* formatter, char* restrict format, ...)
	__attribute__((format(printf, 2, 3)));
void prepend_linear_formatter_str(LinearFormatter* formatter, Str* str);

void assign_ownership_to_formatter(LinearFormatter* formatter, Str* str);

int write_linear_formatter_output(LinearFormatter* formatter, bool allow_stdout);
