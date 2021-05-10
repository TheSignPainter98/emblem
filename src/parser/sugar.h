#pragma once

#include <stddef.h>
#include "data/str.h"

typedef struct
{
	Str* call;
	size_t src_len;
} Sugar;

void make_sugar(Sugar* sugar, Str* call, size_t src_len);
void dest_sugar(Sugar* sugar);
