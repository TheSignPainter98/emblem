/**
 * @file lua-constants.h
 * @brief Exposes function for setting-up interface constants in the extension environment
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-env.h"

void ext_set_global_constants(ExtensionState* s);
