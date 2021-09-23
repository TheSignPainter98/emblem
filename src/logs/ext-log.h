/**
 * @file ext-log.h
 * @brief Exposes functions to make core logging functions available to extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext/ext-env.h"

void set_ext_logging_globals(ExtensionState* s);
