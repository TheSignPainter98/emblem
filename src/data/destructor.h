/**
 * @file destructor.h
 * @brief Provides type definition for destructors, functions responsible for finalising and freeing the memory of a given structure
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include "pp/lambda.h"

/**
 * @brief Type of an object destructor
 *
 * @param o Pointer to an object to destroy
 */
typedef void(fun Destructor)(void* o);
