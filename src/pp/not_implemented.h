/**
 * @file not_implemented.h
 * @brief Preprocessor definitions for function-stubs to stymie compiler errors
 * @author Edward Jones
 * @date 2021-09-17
 */
#pragma once

#include <stdio.h>
#include <stdlib.h>

/**
 * @brief Declare a function signature as not-implemented.
 *
 * Do not use in header files
 *
 * @param sig A signature to mark as unimplemented
 *
 * @return An implementation of the signature which prints that the function is missing before exiting unsuccessfully
 */
#define NOT_IMPLEMENTED(sig)                                                                                           \
	_Pragma("GCC diagnostic push") _Pragma("GCC diagnostic ignored \"-Wunused-parameter\"") sig                        \
	{                                                                                                                  \
		fprintf(stderr, "Function '%s' has not been implemented yet, but has been called! Exiting...\n", __func__);    \
		exit(-1);                                                                                                      \
	}                                                                                                                  \
	_Pragma("GCC diagnostic pop")
