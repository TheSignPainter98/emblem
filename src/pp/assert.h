/**
 * @file assert.h
 * @brief Preprocessor definitions for compile-time assertions
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#define ASSERT(c) ((void)sizeof(char[1 - 2 * !(c)]))
