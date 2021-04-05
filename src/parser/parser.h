#pragma once

#include "argp.h"
#include "data/maybe.h"
#include "data/str.h"
#include "doc-struct/ast.h"

void parse_doc(Maybe* mo, Args* args);

void parse_file(Maybe* mo, Args* args, char* fname);
