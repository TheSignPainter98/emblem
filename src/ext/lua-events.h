#pragma once

#include "ext-params.h"

int do_lua_start_event(ExtensionState* s);
int do_lua_iter_start_event(ExtensionState* s);
int do_lua_iter_end_event(ExtensionState* s);
int do_lua_end_event(ExtensionState* s);
