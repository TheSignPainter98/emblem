#pragma once

/**
 * @brief Declare a variable as unused
 *
 * @param x The name of the variable to declare unused
 *
 * @return A statement without effect which makes the compiler believe that `x` is used
 */
#define UNUSED(x) x = x
