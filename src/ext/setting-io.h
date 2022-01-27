#pragma once

#include "ext-env.h"

void set_ext_setting_globals(ExtensionState* s);
void load_arguments(ExtensionEnv* env, List* args);
const char* get_setting(ExtensionEnv* env, const char* name);
int set_setting(ExtensionEnv* env, const char* name, const char* val);
void release_setting(ExtensionEnv* env);
