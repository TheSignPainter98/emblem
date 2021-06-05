#pragma once

#include "ext-env.h"

#define EM_IMPORT_STYLESHEET_FUNC_NAME "stylesheet"
#define STYLER_LP_LOC				   "_em_styler"

void provide_styler(ExtensionEnv* e);
void rescind_styler(ExtensionEnv* e);

int ext_import_stylesheet(ExtensionState* s);
