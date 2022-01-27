#pragma once

#include "ext-env.h"

void set_ext_setting_globals(ExtensionState* s);
const char* get_setting(ExtensionEnv* env, const char* name);
void release_setting(ExtensionEnv* env);
