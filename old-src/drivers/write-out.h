#pragma once

#include "data/list.h"
#include "data/str.h"
#include <stdbool.h>
#include <stdio.h>

int write_output(Str* fmt, Str* stem, bool allow_stdout, List* content);
int write_output_to_path(Str* fname, List* content);
int write_output_to_file(FILE* fp, List* content);
