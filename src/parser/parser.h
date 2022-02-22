/**
 * @file parser.h
 * @brief Exposes document parser at the topmost level
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "argp.h"
#include "data/locked.h"
#include "data/maybe.h"
#include "data/str.h"
#include "doc-struct/ast.h"

void parse_doc(Maybe* mo, Locked* mtNamesList, Args* args, const char* input);
