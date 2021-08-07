#pragma once

#include "argp.h"
#include "data/locked.h"
#include "data/maybe.h"
#include "data/str.h"
#include "doc-struct/ast.h"

void parse_doc(Maybe* mo, Locked* mtNamesList, Args* args);
