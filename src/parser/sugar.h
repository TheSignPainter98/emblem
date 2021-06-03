#pragma once

#include "data/str.h"
#include <stddef.h>

typedef struct
{
	Str* call;
	size_t src_len;
} Sugar;

void make_sugar(Sugar* sugar, Str* call, size_t src_len);
void dest_sugar(Sugar* sugar);
