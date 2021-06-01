#pragma once

#include "pp/lambda.h"

/**
 * @brief Type of an object destructor
 *
 * @param o Pointer to an object to destroy
 */
typedef void(fun Destructor)(void* o);
