#pragma once

#if __GNUC__
#	define ASSERT(c) ((void)sizeof(char[1 - 2 * !(c)]))
#endif
