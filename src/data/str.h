#pragma once

#include "array.h"
#include "maybe.h"
#include <stdbool.h>
#include <stddef.h>

typedef struct
{
	char* const str;
	size_t const len;
	bool const free_mem;
} Str;

void make_str(Str* str);

void make_strv(Str* str, char* raw);

bool make_strl(Str* str, size_t len);

void dest_str(Str* str);

void str_to_arr(Array* arr, Str* str);

void arr_to_str(Str* str, Array* arr);

void get_strc(Maybe* ret, Str* str, size_t idx);

bool set_strc(Str* str, size_t idx, char val);

bool copy_into_str(Str* cont, Str* ins, size_t startIdx);
