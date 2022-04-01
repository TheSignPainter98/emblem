/**
 * @file shared-destruction.h
 * @brief Declarations for shared destruction
 * @author Edward Jones
 * @date 2022-03-13
 */
#pragma once

typedef enum
{
	CORE_POINTER_DEREFERENCE = 0,
	LUA_POINTER_DEREFERENCE,
} SharedDestructionMode;
