/**
 * @file lua-events.h
 * @brief Exposes functions for issuing typesetting events to extension-space
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "ext-env.h"

int do_ext_start_event(ExtensionState* s);
int do_ext_iter_start_event(ExtensionState* s);
int do_ext_iter_end_event(ExtensionState* s);
int do_ext_end_event(ExtensionState* s);
