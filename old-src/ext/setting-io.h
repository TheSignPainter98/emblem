#pragma once

#include "ext-env.h"

void register_ext_setting(ExtensionState* s);
void load_arguments(ExtensionEnv* env, List* args);
const char* get_setting(ExtensionEnv* env, const char* name);
int set_setting(ExtensionEnv* env, const char* name, const char* val);
void release_setting(ExtensionEnv* env);
