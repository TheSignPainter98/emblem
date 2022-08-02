/**
 * @file ext-loader.h
 * @brief Exposes the extension loader
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-params.h"
#include "ext-env.h"

int load_extensions(ExtensionState* s, ExtParams* params);
