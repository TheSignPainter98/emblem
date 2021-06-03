#include "lambda.h"

#if __clang__
void(fun freel)(void*) = ilambda(void, (void* v), { free(v); });
#elif __GNUC__
void(fun freel)(void*) = free;
#endif
