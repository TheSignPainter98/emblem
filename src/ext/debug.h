/**
 * @file debug.h
 * @brief Exposes functions for debugging the Lua API stack
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include <lua.h>

#ifdef DEBUG
#define dumpstack _dumpstack
#else
#define dumpstack(s)
#endif

void _dumpstack(lua_State* L);
