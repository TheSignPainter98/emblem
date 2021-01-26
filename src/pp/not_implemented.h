#pragma once

#include <stdio.h>
#include <stdlib.h>

#define NOT_IMPLEMENTED(sig)                                                                                           \
	_Pragma("GCC diagnostic push") _Pragma("GCC diagnostic ignored \"-Wunused-parameter\"") sig                        \
	{                                                                                                                  \
		fprintf(stderr, "Function '%s' has not been implemented yet, but has been called! Exiting...\n", __func__);    \
		exit(-1);                                                                                                      \
	}                                                                                                                  \
	_Pragma("GCC diagnostic pop")
