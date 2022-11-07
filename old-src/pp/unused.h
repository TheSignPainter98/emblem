/**
 * @file unused.h
 * @brief Provides preprocessor definitions to declare variables as unused to stymie compiler warnings
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

/**
 * @brief Declare a variable as unused
 *
 * @param x The name of the variable to declare unused
 *
 * @return A statement without effect which makes the compiler believe that `x` is used
 */
#define UNUSED(x) x = x
