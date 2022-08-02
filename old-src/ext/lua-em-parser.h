/**
 * @file lua-em-parser.h
 * @brief Exposes interface between core Emblem parser and extension space
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-env.h"

#define EM_INCLUDE_FILE_FUNC_NAME "__include_file"

int ext_include_file(ExtensionState* s);
