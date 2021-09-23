/**
 * @file lambda.c
 * @brief Implementations of free functions as lambdas
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "lambda.h"

#if __clang__
void(fun freel)(void*) = ilambda(void, (void* v), { free(v); });
#elif __GNUC__
void(fun freel)(void*) = free;
#endif
