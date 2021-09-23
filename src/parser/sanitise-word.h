/**
 * @file sanitise-word.h
 * @brief Exposes word-sanitiser for the Emblem parser
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "data/str.h"
#include "doc-struct/location.h"
#include "emblem-parser.h"
#include <stddef.h>

char* sanitise_word(EM_LTYPE* loc, Str* ifn, char* word, size_t len) __attribute__((hot));
