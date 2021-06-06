#pragma once

#include "data/list.h"
#include "data/map.h"
#include "data/str.h"
#include "driver-params.h"

typedef struct
{
	List* content;
	List* formatter_content;
	Str* output_doc_name;
	Str* stylesheet_name;
	Map* call_name_map;
} LinearFormatter;

void make_linear_formatter(LinearFormatter* formatter, DriverParams* params, size_t num_special_functions,
	const Pair special_functions[num_special_functions], Str* document_output_name_fmt);
void dest_linear_formatter(LinearFormatter* formatter);

void append_linear_formatter_raw(LinearFormatter* formatter, char* raw);
void append_linear_formatter_str(LinearFormatter* formatter, Str* str);
void append_linear_formatter_strf(LinearFormatter* formatter, char* restrict format, ...)
	__attribute__((format(printf, 2, 3)));

int write_linear_formatter_output(LinearFormatter* formatter);
