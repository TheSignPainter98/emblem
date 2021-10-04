/**
 * @file style.h
 * @brief Exposes function for importing stylesheets from extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-env.h"

#define EM_IMPORT_STYLESHEET_FUNC_NAME "stylesheet"
#define STYLER_LP_LOC				   "_em_styler"

int ext_import_stylesheet(ExtensionState* s);
