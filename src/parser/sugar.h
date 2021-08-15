#pragma once

#include "data/str.h"
#include "doc-struct/ast.h"
#include <stddef.h>

typedef struct
{
	Str* call;
	size_t src_len;
} Sugar;

typedef struct
{
	Str* call;
	Str* arg;
} SimpleSugar;

void make_sugar(Sugar* sugar, Str* call, size_t src_len);
void make_sugarv(Sugar* sugar, char* call, size_t src_len);
void dest_sugar(Sugar* sugar);

void make_simple_sugar(SimpleSugar* ssugar, Str* call, Str* arg);
void make_simple_sugarvc(SimpleSugar* ssugar, char* call, char* arg);
void dest_simple_sugar(SimpleSugar* ssugar);
