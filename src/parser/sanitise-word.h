/**
 * @file sanitise-word.h
 * @brief Exposes word-sanitiser for the Emblem parser
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "doc-struct/ast.h"
#include "doc-struct/location.h"

void sanitise_word(Word* word, Location* loc) __attribute__((hot));
